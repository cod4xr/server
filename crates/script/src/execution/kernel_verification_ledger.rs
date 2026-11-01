// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;

use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelFunctionLayout, KernelInstruction, KernelProgramUsage,
    KernelResource, KernelVerificationBudget,
};

pub(super) struct KernelVerificationLedger {
    budget: KernelVerificationBudget,
    pub(super) usage: KernelProgramUsage,
}

impl KernelVerificationLedger {
    pub(super) fn new(
        count: usize,
        layout: KernelFunctionLayout,
        budget: KernelVerificationBudget,
    ) -> Result<Self, KernelError> {
        let count = count as u64;
        kernel_error::check(
            KernelResource::Instructions,
            count,
            budget.max_instructions,
            None,
        )?;
        let slot_bytes = mem::size_of::<KernelInstruction>()
            .checked_add(mem::size_of::<Option<usize>>())
            .and_then(|size| size.checked_add(mem::size_of::<usize>()))
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        let metadata = count
            .checked_mul(slot_bytes as u64)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        kernel_error::check(
            KernelResource::VerificationMetadataBytes,
            metadata,
            budget.max_metadata_bytes,
            None,
        )?;
        let code_bytes = count
            .checked_mul(mem::size_of::<KernelInstruction>() as u64)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        Ok(Self {
            budget,
            usage: KernelProgramUsage {
                string_literals: 0,
                retained_string_bytes: 0,
                parameters: layout.parameters(),
                local_values: layout.local_values(),
                instructions: count,
                verification_steps: 0,
                max_stack_values: 0,
                retained_code_bytes: code_bytes,
                verification_metadata_bytes: metadata,
            },
        })
    }

    pub(super) fn visit(&mut self, pc: usize) -> Result<(), KernelError> {
        self.charge(1, Some(pc as u64))
    }

    pub(super) fn charge(
        &mut self,
        amount: u64,
        instruction: Option<u64>,
    ) -> Result<(), KernelError> {
        let next = self
            .usage
            .verification_steps
            .checked_add(amount)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        kernel_error::check(
            KernelResource::VerificationSteps,
            next,
            self.budget.max_steps,
            instruction,
        )?;
        self.usage.verification_steps = next;
        Ok(())
    }

    pub(super) fn retain_strings(&mut self, count: u64, bytes: u64) -> Result<(), KernelError> {
        let total = self
            .usage
            .verification_metadata_bytes
            .checked_add(bytes)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        kernel_error::check(
            KernelResource::VerificationMetadataBytes,
            total,
            self.budget.max_metadata_bytes,
            None,
        )?;
        self.usage.verification_metadata_bytes = total;
        self.usage.string_literals = count;
        self.usage.retained_string_bytes = bytes;
        Ok(())
    }

    pub(super) fn admit_stack(&mut self, height: usize, pc: usize) -> Result<(), KernelError> {
        kernel_error::check(
            KernelResource::StackValues,
            height as u64,
            self.budget.max_stack_values,
            Some(pc as u64),
        )?;
        self.usage.max_stack_values = self.usage.max_stack_values.max(height as u64);
        Ok(())
    }
}
