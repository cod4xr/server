// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_operations;
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{
    KernelBranchCondition, KernelError, KernelErrorKind, KernelExecutionBudget, KernelInstruction,
    KernelOutcome, KernelResource, VerifiedKernelProgram,
};

pub(super) fn execute(
    program: &VerifiedKernelProgram,
    budget: KernelExecutionBudget,
) -> Result<KernelOutcome, KernelError> {
    if budget.max_steps == 0 {
        kernel_error::check(KernelResource::ExecutionSteps, 1, 0, Some(0))?;
    }
    let mut machine = KernelMachine {
        stack: KernelStack::new(program.usage.max_stack_values, budget)?,
        pc: 0,
        steps: 0,
    };
    machine
        .run(program, budget)
        .map_err(|error| error.with_usage(machine.steps, machine.stack.peak()))
}

struct KernelMachine {
    stack: KernelStack,
    pc: usize,
    steps: u64,
}

impl KernelMachine {
    fn run(
        &mut self,
        program: &VerifiedKernelProgram,
        budget: KernelExecutionBudget,
    ) -> Result<KernelOutcome, KernelError> {
        loop {
            let pc = self.pc;
            let next = self
                .steps
                .checked_add(1)
                .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::SizeOverflow))?;
            kernel_error::check(
                KernelResource::ExecutionSteps,
                next,
                budget.max_steps,
                Some(pc as u64),
            )?;
            self.steps = next;
            let instruction = program
                .instructions
                .get(pc)
                .copied()
                .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
            match instruction {
                KernelInstruction::Return => {
                    let value = self.stack.pop(pc)?;
                    if !self.stack.empty() {
                        return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState));
                    }
                    return Ok(KernelOutcome {
                        value,
                        steps: self.steps,
                        peak_stack_values: self.stack.peak(),
                    });
                }
                KernelInstruction::Jump(target) => self.pc = target,
                KernelInstruction::Branch { condition, target } => {
                    let truth = self.stack.number(pc)?.as_bool();
                    let take = match condition {
                        KernelBranchCondition::IfFalse => !truth,
                        KernelBranchCondition::IfTrue => truth,
                    };
                    self.pc = if take { target } else { advance(pc)? };
                }
                _ => {
                    kernel_operations::apply(instruction, &mut self.stack, pc)?;
                    self.pc = advance(pc)?;
                }
            }
        }
    }
}

fn advance(pc: usize) -> Result<usize, KernelError> {
    pc.checked_add(1)
        .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::SizeOverflow))
}
