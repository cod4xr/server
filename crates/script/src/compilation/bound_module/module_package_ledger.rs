// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_metadata;
use crate::compilation::function_compile_error;
use crate::compilation::{
    FunctionCompileError, FunctionCompileErrorKind, ModuleCompilationBudget,
    ModuleCompilationResource, ModuleCompilationUsage,
};

impl ModuleCompilationLedger {
    pub(super) fn aggregate(
        usage: ModuleCompilationUsage,
        budget: ModuleCompilationBudget,
    ) -> Result<Self, FunctionCompileError> {
        let mut ledger = Self {
            budget,
            usage,
            base_bytes: 0,
        };
        for (resource, requested, maximum) in [
            (
                ModuleCompilationResource::Declarations,
                usage.declarations,
                budget.max_declarations,
            ),
            (
                ModuleCompilationResource::Functions,
                usage.functions,
                budget.max_functions,
            ),
            (
                ModuleCompilationResource::Steps,
                usage.compiler_steps,
                budget.max_steps,
            ),
        ] {
            ledger.check(resource, requested, maximum, 0..0)?;
        }
        ledger.base_bytes = module_metadata::base(usage.declarations, usage.functions, 0..0)?;
        ledger.metadata(ledger.base_bytes, 0..0)?;
        Ok(ledger)
    }

    pub(super) fn extend_base(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let next = self.base_bytes.checked_add(bytes).ok_or_else(|| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
        })?;
        self.temporary(bytes, span)?;
        self.base_bytes = next;
        Ok(())
    }

    pub(super) fn temporary(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let next = self
            .prefix(span.clone())?
            .checked_add(bytes)
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        self.metadata(next, span)
    }
}
