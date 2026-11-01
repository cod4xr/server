// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::calls::{
    KernelFunctionDefinition, KernelModuleUsage, KernelModuleVerificationBudget,
};
use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelInstruction, KernelProgramUsage, KernelResource,
    KernelVerificationBudget, VerifiedKernelProgram,
};

pub(super) struct KernelModuleAdmission {
    pub(super) usage: KernelModuleUsage,
    budget: KernelModuleVerificationBudget,
}

impl KernelModuleAdmission {
    pub(super) fn new(
        definitions: &[KernelFunctionDefinition<'_>],
        budget: KernelModuleVerificationBudget,
    ) -> Result<Self, KernelError> {
        let count = definitions.len() as u64;
        kernel_error::check(KernelResource::Functions, count, budget.max_functions, None)?;
        kernel_error::check(
            KernelResource::VerificationSteps,
            count,
            budget.max_steps,
            None,
        )?;
        let table = bytes(count, size_of::<VerifiedKernelProgram>())?;
        kernel_error::check(
            KernelResource::VerificationMetadataBytes,
            table,
            budget.max_metadata_bytes,
            None,
        )?;
        let mut instructions = 0_u64;
        for (index, definition) in definitions.iter().enumerate() {
            let work = index as u64 + 1;
            instructions = add(instructions, definition.instructions.len() as u64)
                .map_err(|error| error.with_usage(work, 0))?;
            kernel_error::check(
                KernelResource::Instructions,
                instructions,
                budget.max_instructions,
                None,
            )
            .map_err(|error| error.with_usage(work, 0))?;
        }
        let retained = bytes(instructions, size_of::<KernelInstruction>())
            .and_then(|code| add(table, code))
            .map_err(|error| error.with_usage(count, 0))?;
        kernel_error::check(
            KernelResource::VerificationMetadataBytes,
            retained,
            budget.max_metadata_bytes,
            None,
        )
        .map_err(|error| error.with_usage(count, 0))?;
        Ok(Self {
            budget,
            usage: KernelModuleUsage {
                functions: count,
                instructions,
                verification_steps: count,
                retained_bytes: retained,
                verification_metadata_bytes: retained,
            },
        })
    }

    pub(super) fn function_budget(
        &self,
        count: usize,
    ) -> Result<KernelVerificationBudget, KernelError> {
        let prefix = self.metadata_prefix(count)?;
        Ok(KernelVerificationBudget {
            max_instructions: self.budget.max_instructions,
            max_steps: self.budget.max_steps - self.usage.verification_steps,
            max_stack_values: self.budget.max_function_stack_values,
            max_local_values: self.budget.max_function_local_values,
            max_metadata_bytes: self.budget.max_metadata_bytes - prefix,
        })
    }

    pub(super) fn record(&mut self, usage: KernelProgramUsage) -> Result<(), KernelError> {
        let prefix = self
            .usage
            .retained_bytes
            .checked_sub(usage.retained_code_bytes)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::InvalidExecutionState))?;
        let peak = add(prefix, usage.verification_metadata_bytes)?;
        self.usage.verification_steps =
            add(self.usage.verification_steps, usage.verification_steps)?;
        self.usage.verification_metadata_bytes = self.usage.verification_metadata_bytes.max(peak);
        Ok(())
    }

    pub(super) fn error(&self, mut error: KernelError, count: usize) -> KernelError {
        let Some(work) = self.usage.verification_steps.checked_add(error.work_steps) else {
            error.kind = KernelErrorKind::SizeOverflow;
            return error;
        };
        error.work_steps = work;
        if let KernelErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum: _,
        } = error.kind
        {
            let offset = match resource {
                KernelResource::VerificationSteps => {
                    Some((self.usage.verification_steps, self.budget.max_steps))
                }
                KernelResource::VerificationMetadataBytes => match self.metadata_prefix(count) {
                    Ok(prefix) => Some((prefix, self.budget.max_metadata_bytes)),
                    Err(failure) => {
                        error.kind = failure.kind;
                        return error;
                    }
                },
                _ => None,
            };
            if let Some((prefix, maximum)) = offset {
                if let Some(requested) = prefix.checked_add(requested) {
                    error.kind = KernelErrorKind::BudgetExceeded {
                        resource,
                        requested,
                        maximum,
                    };
                } else {
                    error.kind = KernelErrorKind::SizeOverflow;
                }
            }
        }
        error
    }

    fn metadata_prefix(&self, count: usize) -> Result<u64, KernelError> {
        self.usage
            .retained_bytes
            .checked_sub(bytes(count as u64, size_of::<KernelInstruction>())?)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::InvalidExecutionState))
    }
}

fn bytes(count: u64, size: usize) -> Result<u64, KernelError> {
    count
        .checked_mul(size as u64)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))
}

fn add(left: u64, right: u64) -> Result<u64, KernelError> {
    left.checked_add(right)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))
}
