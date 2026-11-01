// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::calls::{
    KernelFunctionDefinition, KernelModuleUsage, KernelModuleVerificationBudget,
};
use crate::execution::kernel_error;
use crate::execution::kernel_literals;
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
        let mut string_literals = 0_u64;
        let mut string_bytes = 0_u64;
        let mut work = 0;
        for definition in definitions {
            // Prepay this table's initial scan and its later metadata-prefix scan.
            let visits = (definition.string_literals.len() as u64)
                .checked_mul(2)
                .ok_or_else(|| {
                    kernel_error::general(KernelErrorKind::SizeOverflow).with_usage(work, 0)
                })?;
            let next = add(work, add(visits, 1)?).map_err(|error| error.with_usage(work, 0))?;
            kernel_error::check(
                KernelResource::VerificationSteps,
                next,
                budget.max_steps,
                None,
            )
            .map_err(|error| error.with_usage(work, 0))?;
            work = next;
            string_literals = add(string_literals, definition.string_literals.len() as u64)
                .map_err(|error| error.with_usage(work, 0))?;
            string_bytes = add(
                string_bytes,
                kernel_literals::retained_bytes(definition.string_literals)
                    .map_err(|error| error.with_usage(work, 0))?,
            )
            .map_err(|error| error.with_usage(work, 0))?;
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
            .and_then(|bytes| add(bytes, string_bytes))
            .map_err(|error| error.with_usage(work, 0))?;
        kernel_error::check(
            KernelResource::VerificationMetadataBytes,
            retained,
            budget.max_metadata_bytes,
            None,
        )
        .map_err(|error| error.with_usage(work, 0))?;
        Ok(Self {
            budget,
            usage: KernelModuleUsage {
                string_literals,
                retained_string_bytes: string_bytes,
                functions: count,
                instructions,
                verification_steps: work,
                retained_bytes: retained,
                verification_metadata_bytes: retained,
            },
        })
    }

    pub(super) fn function_budget(
        &self,
        definition: KernelFunctionDefinition<'_>,
    ) -> Result<KernelVerificationBudget, KernelError> {
        let prefix = self.metadata_prefix(definition)?;
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
            .checked_sub(add(usage.retained_code_bytes, usage.retained_string_bytes)?)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::InvalidExecutionState))?;
        let peak = add(prefix, usage.verification_metadata_bytes)?;
        self.usage.verification_steps =
            add(self.usage.verification_steps, usage.verification_steps)?;
        self.usage.verification_metadata_bytes = self.usage.verification_metadata_bytes.max(peak);
        Ok(())
    }

    pub(super) fn error(
        &self,
        mut error: KernelError,
        budget: KernelVerificationBudget,
    ) -> KernelError {
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
                KernelResource::VerificationMetadataBytes => Some((
                    self.budget.max_metadata_bytes - budget.max_metadata_bytes,
                    self.budget.max_metadata_bytes,
                )),
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

    fn metadata_prefix(
        &self,
        definition: KernelFunctionDefinition<'_>,
    ) -> Result<u64, KernelError> {
        self.usage
            .retained_bytes
            .checked_sub(add(
                bytes(
                    definition.instructions.len() as u64,
                    size_of::<KernelInstruction>(),
                )?,
                kernel_literals::retained_bytes(definition.string_literals)?,
            )?)
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
