// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_frame::{KernelFrame, KernelFrameStep};
use crate::execution::kernel_locals::{KernelArgumentBinding, KernelLocalAdmission};
use crate::execution::{
    KernelError, KernelErrorKind, KernelExecutionBudget, KernelOutcome, KernelResource,
    KernelValue, VerifiedKernelProgram,
};

pub(super) fn execute(
    program: &VerifiedKernelProgram,
    arguments: &[KernelValue],
    budget: KernelExecutionBudget,
    binding: KernelArgumentBinding,
) -> Result<KernelOutcome, KernelError> {
    if budget.max_steps == 0 {
        kernel_error::check(KernelResource::ExecutionSteps, 1, 0, Some(0))?;
    }
    let admission = KernelLocalAdmission::admit(program.layout, arguments, budget, binding)?;
    let local_values = admission.count();
    execute_admitted(program, arguments, budget, admission)
        .map_err(|error| error.with_locals(local_values))
}

fn execute_admitted(
    program: &VerifiedKernelProgram,
    arguments: &[KernelValue],
    budget: KernelExecutionBudget,
    admission: KernelLocalAdmission,
) -> Result<KernelOutcome, KernelError> {
    let mut frame = KernelFrame::from_admission(program, arguments, budget, admission)?;
    let mut steps = 0;
    loop {
        let step = frame
            .step(program, &mut steps, budget.max_steps, None)
            .map_err(|error| error.with_usage(steps, frame.stack.peak()))?;
        match step {
            KernelFrameStep::Return(value) => {
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
