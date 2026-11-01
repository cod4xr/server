// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::binding::{LegacyFunctionInclusion, LegacyModuleSymbols};
use crate::compilation::bound_module::module_metadata;
use crate::compilation::bound_module::{
    ModuleCompilationBudget, ModuleCompilationResource, ModuleCompilationUsage,
};
use crate::compilation::function_compile_error;
use crate::compilation::{
    FunctionCompilationResource, FunctionCompilationUsage, FunctionCompileError,
    FunctionCompileErrorKind, FunctionLoweringBudget,
};

pub(super) struct ModuleCompilationLedger {
    pub(super) budget: ModuleCompilationBudget,
    pub(super) usage: ModuleCompilationUsage,
    base_bytes: u64,
}

impl ModuleCompilationLedger {
    pub(super) fn new(
        symbols: &LegacyModuleSymbols<'_, '_, '_>,
        budget: ModuleCompilationBudget,
        span: Range<u64>,
    ) -> Result<Self, FunctionCompileError> {
        let mut ledger = Self {
            budget,
            usage: ModuleCompilationUsage {
                declarations: symbols.functions().len() as u64,
                ..ModuleCompilationUsage::default()
            },
            base_bytes: 0,
        };
        ledger.check(
            ModuleCompilationResource::Declarations,
            ledger.usage.declarations,
            budget.max_declarations,
            span.clone(),
        )?;
        for binding in symbols.functions() {
            ledger.charge(1, binding.declaration().span())?;
            if binding.inclusion() == LegacyFunctionInclusion::Active {
                ledger.usage.functions = add(ledger.usage.functions, 1, span.clone())?;
                ledger.check(
                    ModuleCompilationResource::Functions,
                    ledger.usage.functions,
                    budget.max_functions,
                    binding.declaration().span(),
                )?;
            }
        }
        ledger.base_bytes = module_metadata::base(
            ledger.usage.declarations,
            ledger.usage.functions,
            span.clone(),
        )?;
        ledger.metadata(ledger.base_bytes, span)?;
        Ok(ledger)
    }

    pub(super) fn charge(
        &mut self,
        amount: u64,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let next = add(self.usage.compiler_steps, amount, span.clone())?;
        self.check(
            ModuleCompilationResource::Steps,
            next,
            self.budget.max_steps,
            span,
        )?;
        self.usage.compiler_steps = next;
        Ok(())
    }

    pub(super) fn function_budget(
        &self,
        span: Range<u64>,
    ) -> Result<FunctionLoweringBudget, FunctionCompileError> {
        let mut budget = self.budget.function;
        budget.max_steps = budget
            .max_steps
            .min(self.budget.max_steps - self.usage.compiler_steps);
        budget.max_instructions = budget
            .max_instructions
            .min(self.budget.max_instructions - self.usage.instructions);
        budget.max_metadata_bytes = budget
            .max_metadata_bytes
            .min(self.budget.max_metadata_bytes - self.prefix(span)?);
        Ok(budget)
    }

    pub(super) fn record(
        &mut self,
        usage: FunctionCompilationUsage,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let peak = add(
            self.prefix(span.clone())?,
            usage.peak_metadata_bytes,
            span.clone(),
        )?;
        self.metadata(peak, span.clone())?;
        self.charge(usage.compiler_steps, span.clone())?;
        let instructions = add(self.usage.instructions, usage.instructions, span.clone())?;
        self.check(
            ModuleCompilationResource::Instructions,
            instructions,
            self.budget.max_instructions,
            span,
        )?;
        self.usage.instructions = instructions;
        Ok(())
    }

    pub(super) fn locate(&self, mut cause: FunctionCompileError) -> FunctionCompileError {
        cause.work_steps = self.usage.compiler_steps;
        cause.peak_metadata_bytes = self.usage.peak_metadata_bytes;
        cause
    }

    pub(super) fn function_error(&self, mut cause: FunctionCompileError) -> FunctionCompileError {
        let span = cause.span.clone();
        let result = self.translate_failure(&mut cause).and_then(|()| {
            cause.work_steps = add(self.usage.compiler_steps, cause.work_steps, span.clone())?;
            cause.peak_metadata_bytes = self.usage.peak_metadata_bytes.max(add(
                self.prefix(span.clone())?,
                cause.peak_metadata_bytes,
                span,
            )?);
            Ok(())
        });
        if let Err(failure) = result {
            cause.kind = failure.kind;
        }
        cause
    }

    pub(super) fn retained_metadata(&self, span: Range<u64>) -> Result<u64, FunctionCompileError> {
        module_metadata::retained(
            self.usage.declarations,
            self.usage.functions,
            self.usage.instructions,
            span,
        )
    }

    fn translate_failure(
        &self,
        cause: &mut FunctionCompileError,
    ) -> Result<(), FunctionCompileError> {
        let FunctionCompileErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum: _,
        } = cause.kind
        else {
            return Ok(());
        };
        let span = cause.span.clone();
        let candidate = match resource {
            FunctionCompilationResource::Steps => Some((
                ModuleCompilationResource::Steps,
                self.usage.compiler_steps,
                self.budget.max_steps,
                self.budget.function.max_steps,
            )),
            FunctionCompilationResource::Instructions => Some((
                ModuleCompilationResource::Instructions,
                self.usage.instructions,
                self.budget.max_instructions,
                self.budget.function.max_instructions,
            )),
            FunctionCompilationResource::MetadataBytes => Some((
                ModuleCompilationResource::MetadataBytes,
                self.prefix(span.clone())?,
                self.budget.max_metadata_bytes,
                self.budget.function.max_metadata_bytes,
            )),
            _ => None,
        };
        if let Some((resource, prefix, maximum, function_maximum)) = candidate
            && maximum - prefix <= function_maximum
        {
            cause.kind = FunctionCompileErrorKind::ModuleBudgetExceeded {
                resource,
                requested: add(prefix, requested, span)?,
                maximum,
            };
        }
        Ok(())
    }

    fn prefix(&self, span: Range<u64>) -> Result<u64, FunctionCompileError> {
        add(
            self.base_bytes,
            module_metadata::staged(self.usage.instructions, span.clone())?,
            span,
        )
    }

    fn metadata(&mut self, requested: u64, span: Range<u64>) -> Result<(), FunctionCompileError> {
        self.check(
            ModuleCompilationResource::MetadataBytes,
            requested,
            self.budget.max_metadata_bytes,
            span,
        )?;
        self.usage.peak_metadata_bytes = self.usage.peak_metadata_bytes.max(requested);
        Ok(())
    }

    fn check(
        &self,
        resource: ModuleCompilationResource,
        requested: u64,
        maximum: u64,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        if requested > maximum {
            return Err(self.locate(function_compile_error::at(
                span,
                FunctionCompileErrorKind::ModuleBudgetExceeded {
                    resource,
                    requested,
                    maximum,
                },
            )));
        }
        Ok(())
    }
}

fn add(left: u64, right: u64, span: Range<u64>) -> Result<u64, FunctionCompileError> {
    left.checked_add(right)
        .ok_or_else(|| function_compile_error::at(span, FunctionCompileErrorKind::SizeOverflow))
}
