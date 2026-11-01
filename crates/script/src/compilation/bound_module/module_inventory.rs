// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;
use std::ops::Range;

use crate::binding::{LegacyFunctionInclusion, LegacyModuleSymbols};
use crate::compilation::FunctionCompileErrorKind;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::{CompiledModuleDeclaration, ModuleCompileError};
use crate::compilation::function_call_scope::{
    FunctionCallName, FunctionCallOrigin, FunctionCallScope, FunctionCallTarget,
};
use crate::compilation::function_compile_error;

pub(super) struct ModuleInventory<'source> {
    pub(super) scope: FunctionCallScope<'source>,
    pub(super) declarations: Vec<CompiledModuleDeclaration>,
}

pub(super) fn build<'source>(
    symbols: &LegacyModuleSymbols<'_, '_, 'source>,
    ledger: &mut ModuleCompilationLedger,
    span: Range<u64>,
) -> Result<ModuleInventory<'source>, ModuleCompileError> {
    let mut declarations = module_storage::reserve(ledger.usage.declarations, span.clone())
        .map_err(|cause| ModuleCompileError::at(None, ledger.locate(cause)))?;
    let mut scope = FunctionCallScope {
        names: HashMap::new(),
        modules: None,
        largest_name_bytes: 0,
    };
    let count = usize::try_from(ledger.usage.functions).map_err(|_| {
        ModuleCompileError::at(
            None,
            ledger.locate(function_compile_error::at(
                span.clone(),
                FunctionCompileErrorKind::SizeOverflow,
            )),
        )
    })?;
    scope.names.try_reserve(count).map_err(|_| {
        ModuleCompileError::at(
            None,
            ledger.locate(function_compile_error::at(
                span,
                FunctionCompileErrorKind::AllocationFailed,
            )),
        )
    })?;
    for (index, binding) in symbols.functions().iter().copied().enumerate() {
        let declaration = binding.declaration();
        ledger
            .charge(1, declaration.span())
            .map_err(|cause| ModuleCompileError::at(Some(index), cause))?;
        let function_index = if binding.inclusion() == LegacyFunctionInclusion::Active {
            let name = binding.name();
            let bytes = name.as_bytes().len() as u64;
            ledger
                .charge(bytes, name.lexeme().span())
                .map_err(|cause| ModuleCompileError::at(Some(index), cause))?;
            let function = scope.names.len();
            if scope
                .names
                .insert(
                    FunctionCallName { module: 0, name },
                    FunctionCallTarget {
                        origin: FunctionCallOrigin::Direct,
                        function,
                        visibility: binding.visibility(),
                    },
                )
                .is_some()
            {
                return Err(ModuleCompileError::at(
                    Some(index),
                    ledger.locate(function_compile_error::at(
                        name.lexeme().span(),
                        FunctionCompileErrorKind::InvalidStructure,
                    )),
                ));
            }
            scope.largest_name_bytes = scope.largest_name_bytes.max(bytes);
            Some(function as u64)
        } else {
            None
        };
        declarations.push(CompiledModuleDeclaration {
            span: declaration.span(),
            parameters: declaration.parameters().len() as u64,
            visibility: binding.visibility(),
            inclusion: binding.inclusion(),
            function_index,
        });
    }
    Ok(ModuleInventory {
        scope,
        declarations,
    })
}
