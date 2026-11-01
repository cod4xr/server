// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::calls::kernel_call_frame::KernelCallFrame;
use crate::execution::calls::kernel_call_ledger::KernelCallLedger;
use crate::execution::calls::{
    KernelCallBudget, KernelCallOutcome, KernelModuleError, VerifiedKernelModule,
};
use crate::execution::heap;
use crate::execution::kernel_error;
use crate::execution::kernel_frame::{KernelFrame, KernelFrameStep};
use crate::execution::kernel_frame_identity::KernelFrameIdentity;
use crate::execution::kernel_locals::KernelArgumentBinding;
use crate::execution::{
    KernelError, KernelErrorKind, KernelHeapContext, KernelResource, KernelValue,
    VerifiedKernelProgram,
};

pub(super) fn execute(
    module: &VerifiedKernelModule,
    entry: u64,
    arguments: &[KernelValue],
    budget: KernelCallBudget,
    mut heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<KernelCallOutcome, KernelModuleError> {
    let result = execute_inner(module, entry, arguments, budget, heap.as_deref_mut());
    if let Some(context) = heap
        && let Err(kind) = context.finish_array_owners()
    {
        return Err(match result {
            Err(mut error) => {
                error.cause.kind = kind;
                error
            }
            Ok(outcome) => {
                let mut ledger = KernelCallLedger::new(budget);
                ledger.usage = outcome.usage;
                ledger.error(None, None, kernel_error::general(kind))
            }
        });
    }
    result
}

fn execute_inner(
    module: &VerifiedKernelModule,
    entry: u64,
    arguments: &[KernelValue],
    budget: KernelCallBudget,
    mut heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<KernelCallOutcome, KernelModuleError> {
    let mut ledger = KernelCallLedger::new(budget);
    let index = usize::try_from(entry).ok();
    let proof = index
        .and_then(|index| module.functions.get(index))
        .ok_or_else(|| {
            ledger.error(
                None,
                None,
                kernel_error::general(KernelErrorKind::InvalidFunction { function: entry }),
            )
        })?;
    let index = index.ok_or_else(|| ledger.error(None, None, state()))?;
    check_arguments(proof, arguments.len())
        .map_err(|cause| ledger.error(Some(index), None, cause))?;
    kernel_error::check(KernelResource::ExecutionSteps, 1, budget.max_steps, Some(0))
        .map_err(|cause| ledger.error(Some(index), None, cause))?;
    let admission = ledger
        .admit(proof, arguments.len())
        .map_err(|cause| ledger.error(Some(index), None, cause))?;
    heap::validate_arguments(arguments, heap.as_deref_mut())
        .map_err(|kind| ledger.error(Some(index), None, kernel_error::general(kind)))?;
    if let Some(heap) = heap.as_deref_mut() {
        heap.prepare_literals(module.usage.string_literals)
            .map_err(|kind| ledger.error(Some(index), None, kernel_error::general(kind)))?;
    }
    let mut frames = Vec::new();
    reserve(&mut frames).map_err(|cause| ledger.error(Some(index), None, cause))?;
    let mut machine = KernelFrame::new(
        proof,
        arguments,
        admission,
        KernelArgumentBinding::LegacyScript,
    )
    .map_err(|cause| ledger.error(Some(index), None, cause))?;
    machine
        .bind_array_arguments(arguments, heap.as_deref_mut())
        .map_err(|cause| ledger.error(Some(index), None, cause))?;
    frames.push(KernelCallFrame {
        function: index,
        machine,
    });
    run(module, &mut frames, &mut ledger, budget.max_steps, heap)
}

fn run(
    module: &VerifiedKernelModule,
    frames: &mut Vec<KernelCallFrame>,
    ledger: &mut KernelCallLedger,
    maximum_steps: u64,
    mut heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<KernelCallOutcome, KernelModuleError> {
    loop {
        let (frame, parents) = frames
            .split_last_mut()
            .ok_or_else(|| ledger.error(None, None, state()))?;
        let function = frame.function;
        let proof = module
            .functions
            .get(function)
            .ok_or_else(|| ledger.error(Some(function), None, state()))?;
        let step = frame
            .machine
            .step(
                proof,
                &mut ledger.usage.steps,
                maximum_steps,
                KernelFrameIdentity {
                    depth: parents.len(),
                    module: Some(module.identity),
                },
                heap.as_deref_mut(),
                parents.iter().flat_map(|parent| parent.machine.values()),
            )
            .map_err(|cause| {
                ledger.error(
                    Some(function),
                    None,
                    cause
                        .with_usage(ledger.usage.steps, frame.machine.stack.peak())
                        .with_locals(proof.layout.local_values()),
                )
            })?;
        match step {
            KernelFrameStep::Continue => {}
            KernelFrameStep::Call {
                function: target,
                arguments,
                pc,
            } => {
                push_call(
                    module,
                    frames,
                    ledger,
                    target,
                    arguments,
                    pc,
                    heap.as_deref_mut(),
                )
                .map_err(|cause| ledger.error(Some(function), Some(pc), cause))?;
            }
            KernelFrameStep::Return(value) => {
                if let Some(context) = heap.as_deref_mut() {
                    let owner = frames
                        .len()
                        .checked_sub(1)
                        .ok_or_else(|| ledger.error(Some(function), None, state()))?;
                    context
                        .return_array_frame(owner, value, owner.checked_sub(1))
                        .map_err(|kind| {
                            ledger.error(Some(function), None, kernel_error::general(kind))
                        })?;
                }
                let finished = frames
                    .pop()
                    .ok_or_else(|| ledger.error(Some(function), None, state()))?;
                ledger
                    .release(proof)
                    .map_err(|cause| ledger.error(Some(function), None, cause))?;
                drop(finished);
                if let Some(caller) = frames.last_mut() {
                    caller
                        .machine
                        .stack
                        .push(value, caller.machine.pc)
                        .map_err(|cause| ledger.error(Some(caller.function), None, cause))?;
                } else {
                    return Ok(KernelCallOutcome {
                        value,
                        usage: ledger.usage,
                    });
                }
            }
        }
    }
}

fn push_call(
    module: &VerifiedKernelModule,
    frames: &mut Vec<KernelCallFrame>,
    ledger: &mut KernelCallLedger,
    function: usize,
    arguments: usize,
    pc: usize,
    mut heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<(), KernelError> {
    let proof: &VerifiedKernelProgram = module.functions.get(function).ok_or_else(state)?;
    check_arguments(proof, arguments)?;
    let admission = ledger.admit(proof, arguments)?;
    reserve(frames)?;
    let mut machine = KernelFrame::new(proof, &[], admission, KernelArgumentBinding::LegacyScript)?;
    let owner = frames.len();
    let parent = owner.checked_sub(1).ok_or_else(state)?;
    let caller = frames.last_mut().ok_or_else(state)?;
    for parameter in 0..arguments {
        let value = caller.machine.stack.pop(pc)?;
        if let KernelValue::Array(id) = value {
            heap.as_deref_mut()
                .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::ObjectExecutionUnsupported))?
                .move_owned_array(parent, owner, id)
                .map_err(|kind| kernel_error::at(pc, kind))?;
        }
        machine.locals.store(parameter, value, pc)?;
    }
    frames.push(KernelCallFrame { function, machine });
    Ok(())
}

fn reserve(frames: &mut Vec<KernelCallFrame>) -> Result<(), KernelError> {
    frames
        .try_reserve_exact(1)
        .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))
}

fn check_arguments(proof: &VerifiedKernelProgram, count: usize) -> Result<(), KernelError> {
    if count as u64 > proof.layout.parameters() {
        return Err(kernel_error::general(KernelErrorKind::ArgumentCount {
            expected: proof.layout.parameters(),
            actual: count as u64,
        }));
    }
    Ok(())
}

fn state() -> KernelError {
    kernel_error::general(KernelErrorKind::InvalidExecutionState)
}
