// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_locals::{KernelArgumentBinding, KernelLocalAdmission, KernelLocals};
use crate::execution::kernel_operations;
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{
    KernelBranchCondition, KernelError, KernelErrorKind, KernelExecutionBudget, KernelInstruction,
    KernelResource, KernelValue, VerifiedKernelProgram,
};

pub(super) struct KernelFrame {
    pub(super) stack: KernelStack,
    pub(super) locals: KernelLocals,
    pub(super) pc: usize,
}

pub(super) enum KernelFrameStep {
    Continue,
    Return(KernelValue),
    Call {
        function: usize,
        arguments: usize,
        pc: usize,
    },
}

impl KernelFrame {
    pub(super) fn new(
        program: &VerifiedKernelProgram,
        arguments: &[KernelValue],
        budget: KernelExecutionBudget,
        binding: KernelArgumentBinding,
    ) -> Result<Self, KernelError> {
        let admission = KernelLocalAdmission::admit(program.layout, arguments, budget, binding)?;
        Self::from_admission(program, arguments, budget, admission)
    }

    pub(super) fn from_admission(
        program: &VerifiedKernelProgram,
        arguments: &[KernelValue],
        budget: KernelExecutionBudget,
        admission: KernelLocalAdmission,
    ) -> Result<Self, KernelError> {
        Ok(Self {
            stack: KernelStack::new(program.usage.max_stack_values, budget)?,
            locals: admission.allocate(arguments)?,
            pc: 0,
        })
    }

    pub(super) fn step(
        &mut self,
        program: &VerifiedKernelProgram,
        steps: &mut u64,
        maximum: u64,
    ) -> Result<KernelFrameStep, KernelError> {
        let pc = self.pc;
        let next = steps
            .checked_add(1)
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::SizeOverflow))?;
        kernel_error::check(
            KernelResource::ExecutionSteps,
            next,
            maximum,
            Some(pc as u64),
        )?;
        *steps = next;
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
                return Ok(KernelFrameStep::Return(value));
            }
            KernelInstruction::Call {
                function,
                arguments,
            } => {
                self.pc = advance(pc)?;
                return Ok(KernelFrameStep::Call {
                    function,
                    arguments,
                    pc,
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
                kernel_operations::apply(instruction, &mut self.stack, &mut self.locals, pc)?;
                self.pc = advance(pc)?;
            }
        }
        Ok(KernelFrameStep::Continue)
    }
}

fn advance(pc: usize) -> Result<usize, KernelError> {
    pc.checked_add(1)
        .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::SizeOverflow))
}
