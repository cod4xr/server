// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap;
use crate::execution::kernel_error;
use crate::execution::kernel_frame::{KernelFrame, KernelFrameStep};
use crate::execution::kernel_frame_identity::KernelFrameIdentity;
use crate::execution::kernel_locals::{KernelArgumentBinding, KernelLocalAdmission};
use crate::execution::{
    KernelError, KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelOutcome,
    KernelResource, KernelValue, VerifiedKernelProgram,
};

pub(super) fn execute(
    program: &VerifiedKernelProgram,
    arguments: &[KernelValue],
    budget: KernelExecutionBudget,
    binding: KernelArgumentBinding,
    mut heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<KernelOutcome, KernelError> {
    let result = execute_inner(program, arguments, budget, binding, heap.as_deref_mut());
    if let Some(context) = heap
        && let Err(kind) = context.finish_array_owners()
    {
        return Err(match result {
            Err(mut error) => {
                error.kind = kind;
                error
            }
            Ok(outcome) => {
                kernel_error::general(kind).with_usage(outcome.steps, outcome.peak_stack_values)
            }
        });
    }
    result
}

fn execute_inner(
    program: &VerifiedKernelProgram,
    arguments: &[KernelValue],
    budget: KernelExecutionBudget,
    binding: KernelArgumentBinding,
    mut heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<KernelOutcome, KernelError> {
    if budget.max_steps == 0 {
        kernel_error::check(KernelResource::ExecutionSteps, 1, 0, Some(0))?;
    }
    let admission = KernelLocalAdmission::admit(program.layout, arguments, budget, binding)?;
    let local_values = admission.count();
    heap::validate_arguments(arguments, heap.as_deref_mut())
        .map_err(|kind| kernel_error::general(kind).with_locals(local_values))?;
    if let Some(heap) = heap.as_deref_mut() {
        heap.prepare_literals(program.usage.string_literals)
            .map_err(|kind| kernel_error::general(kind).with_locals(local_values))?;
    }
    execute_admitted(program, arguments, budget, admission, heap)
        .map_err(|error| error.with_locals(local_values))
}

fn execute_admitted(
    program: &VerifiedKernelProgram,
    arguments: &[KernelValue],
    budget: KernelExecutionBudget,
    admission: KernelLocalAdmission,
    mut heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<KernelOutcome, KernelError> {
    let mut frame = KernelFrame::from_admission(program, arguments, budget, admission)?;
    frame.bind_array_arguments(arguments, heap.as_deref_mut())?;
    let mut steps = 0;
    loop {
        let step = frame
            .step(
                program,
                &mut steps,
                budget.max_steps,
                KernelFrameIdentity {
                    depth: 0,
                    module: None,
                },
                heap.as_deref_mut(),
                std::iter::empty(),
            )
            .map_err(|error| error.with_usage(steps, frame.stack.peak()))?;
        match step {
            KernelFrameStep::Return(value) => {
                if let Some(context) = heap.as_deref_mut() {
                    context.return_array_frame(0, value, None).map_err(|kind| {
                        kernel_error::general(kind).with_usage(steps, frame.stack.peak())
                    })?;
                }
                return Ok(KernelOutcome {
                    value,
                    steps,
                    peak_stack_values: frame.stack.peak(),
                    local_values: frame.locals.count(),
                    local_bytes: frame.locals.bytes(),
                });
            }
            KernelFrameStep::Continue => {}
            KernelFrameStep::Call { pc, .. } => {
                return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState)
                    .with_usage(steps, frame.stack.peak()));
            }
        }
    }
}
