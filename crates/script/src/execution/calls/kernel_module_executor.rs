// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::calls::kernel_call_frame::KernelCallFrame;
use crate::execution::calls::kernel_call_ledger::KernelCallLedger;
use crate::execution::calls::{
    KernelCallBudget, KernelCallOutcome, KernelModuleError, VerifiedKernelModule,
};
use crate::execution::kernel_error;
use crate::execution::kernel_frame::{KernelFrame, KernelFrameStep};
use crate::execution::kernel_locals::KernelArgumentBinding;
use crate::execution::{
    KernelError, KernelErrorKind, KernelResource, KernelValue, VerifiedKernelProgram,
};

pub(super) fn execute(
    module: &VerifiedKernelModule,
    entry: u64,
    arguments: &[KernelValue],
    budget: KernelCallBudget,
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
    if arguments.len() as u64 > proof.layout.parameters() {
        return Err(ledger.error(
            Some(index),
            None,
            kernel_error::general(KernelErrorKind::ArgumentCount {
                expected: proof.layout.parameters(),
                actual: arguments.len() as u64,
            }),
        ));
    }
    kernel_error::check(KernelResource::ExecutionSteps, 1, budget.max_steps, Some(0))
        .map_err(|cause| ledger.error(Some(index), None, cause))?;
    let admission = ledger
        .admit(proof, arguments.len())
        .map_err(|cause| ledger.error(Some(index), None, cause))?;
    let mut frames = Vec::new();
    reserve(&mut frames).map_err(|cause| ledger.error(Some(index), None, cause))?;
    let machine = KernelFrame::new(
        proof,
        arguments,
        admission,
        KernelArgumentBinding::LegacyScript,
    )
    .map_err(|cause| ledger.error(Some(index), None, cause))?;
    frames.push(KernelCallFrame {
        function: index,
        machine,
    });
    run(module, &mut frames, &mut ledger, budget.max_steps)
}

fn run(
    module: &VerifiedKernelModule,
    frames: &mut Vec<KernelCallFrame>,
    ledger: &mut KernelCallLedger,
    maximum_steps: u64,
) -> Result<KernelCallOutcome, KernelModuleError> {
    loop {
        let frame = frames
            .last_mut()
            .ok_or_else(|| ledger.error(None, None, state()))?;
        let function = frame.function;
        let proof = module
            .functions
            .get(function)
            .ok_or_else(|| ledger.error(Some(function), None, state()))?;
        let step = frame
            .machine
            .step(proof, &mut ledger.usage.steps, maximum_steps)
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
                push_call(module, frames, ledger, target, arguments, pc)
                    .map_err(|cause| ledger.error(Some(function), Some(pc), cause))?;
            }
            KernelFrameStep::Return(value) => {
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
) -> Result<(), KernelError> {
    let proof: &VerifiedKernelProgram = module.functions.get(function).ok_or_else(state)?;
    let admission = ledger.admit(proof, arguments)?;
    reserve(frames)?;
    let mut machine = KernelFrame::new(proof, &[], admission, KernelArgumentBinding::LegacyScript)?;
    let caller = frames.last_mut().ok_or_else(state)?;
    for parameter in 0..arguments {
        machine
            .locals
            .store(parameter, caller.machine.stack.pop(pc)?, pc)?;
    }
    frames.push(KernelCallFrame { function, machine });
    Ok(())
}

fn reserve(frames: &mut Vec<KernelCallFrame>) -> Result<(), KernelError> {
    frames
        .try_reserve_exact(1)
        .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))
}

fn state() -> KernelError {
    kernel_error::general(KernelErrorKind::InvalidExecutionState)
}
