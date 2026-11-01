// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::FunctionCompileErrorKind;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::staged_function::StagedFunctionCode;
use crate::compilation::bound_module::{
    CompiledModuleDeclaration, CompiledModuleFunction, ModuleCompileError,
};
use crate::compilation::function_compile_error;
use crate::execution::{KernelFunctionDefinition, KernelModuleError, VerifiedKernelModule};

pub(super) fn verify(
    code: &[StagedFunctionCode],
    maps: &mut [CompiledModuleFunction],
    declarations: &[CompiledModuleDeclaration],
    ledger: &mut ModuleCompilationLedger,
    span: Range<u64>,
) -> Result<VerifiedKernelModule, ModuleCompileError> {
    if code.len() != maps.len() || code.len() as u64 != ledger.usage.functions {
        return Err(ModuleCompileError::at(
            None,
            ledger.locate(function_compile_error::at(
                span,
                FunctionCompileErrorKind::InvalidStructure,
            )),
        ));
    }
    let mut definitions = module_storage::reserve(ledger.usage.functions, span.clone())
        .map_err(|cause| ModuleCompileError::at(None, ledger.locate(cause)))?;
    for function in code {
        ledger
            .charge(1, span.clone())
            .map_err(|cause| ModuleCompileError::at(None, cause))?;
        definitions.push(KernelFunctionDefinition {
            instructions: &function.instructions,
            string_literals: &function.string_literals,
            field_names: &function.field_names,
            switch_labels: &function.switch_labels,
            layout: function.layout,
        });
    }
    let program = VerifiedKernelModule::verify(&definitions, ledger.budget.verification)
        .map_err(|cause| located(cause, maps, declarations, ledger, span.clone()))?;
    for (index, map) in maps.iter_mut().enumerate() {
        ledger
            .charge(1, span.clone())
            .map_err(|cause| ModuleCompileError::at(Some(map.declaration), cause))?;
        let proof = program.function_usage(index as u64).ok_or_else(|| {
            ModuleCompileError::at(
                Some(map.declaration),
                ledger.locate(function_compile_error::at(
                    span.clone(),
                    FunctionCompileErrorKind::InvalidStructure,
                )),
            )
        })?;
        map.usage.verification_steps = proof.verification_steps;
    }
    ledger.usage.verification = program.usage();
    ledger.usage.retained_metadata_bytes = ledger
        .retained_metadata(span)
        .map_err(|cause| ModuleCompileError::at(None, ledger.locate(cause)))?;
    Ok(program)
}

fn located(
    error: KernelModuleError,
    maps: &[CompiledModuleFunction],
    declarations: &[CompiledModuleDeclaration],
    ledger: &ModuleCompilationLedger,
    fallback: Range<u64>,
) -> ModuleCompileError {
    let map = error
        .function
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|index| maps.get(index));
    let declaration = map.map(|map| map.declaration);
    let span = map
        .and_then(|map| {
            error
                .cause
                .instruction
                .and_then(|pc| map.instruction_span(pc))
        })
        .or_else(|| {
            declaration
                .and_then(|index| declarations.get(index))
                .map(|declaration| declaration.span.clone())
        })
        .unwrap_or(fallback);
    ModuleCompileError::at(
        declaration,
        ledger.locate(function_compile_error::at(
            span,
            FunctionCompileErrorKind::Kernel(error.cause),
        )),
    )
}
