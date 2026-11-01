// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

use crate::binding::{CanonicalScriptPath, LegacyFunctionInclusion, ScriptPathBudget};
use crate::compilation::FunctionCompileErrorKind;
use crate::compilation::bound_module::CompiledModuleDeclaration;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::package::{ScriptModuleDefinition, ScriptPackageError};
use crate::compilation::function_call_scope::{
    FunctionCallName, FunctionCallOrigin, FunctionCallScope, FunctionCallTarget,
};
use crate::compilation::function_compile_error;
use crate::compilation::module_call_scope::ModuleCallScope;

pub(super) struct PackageInventory<'scope> {
    pub(super) scope: FunctionCallScope<'scope>,
    pub(super) declarations: Vec<CompiledModuleDeclaration>,
}

pub(super) fn build<'scope, 'module, 'tree, 'source: 'scope>(
    inputs: &[ScriptModuleDefinition<'_, 'module, 'tree, 'source>],
    names: &'scope [CanonicalScriptPath],
    order: &'scope [usize],
    path_budget: ScriptPathBudget,
    aliases: u64,
    ledger: &mut ModuleCompilationLedger,
) -> Result<PackageInventory<'scope>, ScriptPackageError> {
    let mut scope: FunctionCallScope<'scope> = FunctionCallScope {
        names: HashMap::new(),
        largest_name_bytes: 0,
        modules: Some(ModuleCallScope {
            names,
            order,
            path_budget,
        }),
    };
    let targets = ledger
        .usage
        .functions
        .checked_add(aliases)
        .ok_or_else(|| error(None, None, FunctionCompileErrorKind::SizeOverflow, ledger))?;
    let count = usize::try_from(targets)
        .map_err(|_| error(None, None, FunctionCompileErrorKind::SizeOverflow, ledger))?;
    // Reserve direct and imported keys while empty. Growing after direct insertion
    // could rehash borrowed names outside the individual prepaid alias visits.
    scope.names.try_reserve(count).map_err(|_| {
        error(
            None,
            None,
            FunctionCompileErrorKind::AllocationFailed,
            ledger,
        )
    })?;
    let mut declarations = module_storage::reserve(ledger.usage.declarations, 0..0)
        .map_err(|cause| ScriptPackageError::at(None, None, ledger.locate(cause)))?;
    for (module, input) in inputs.iter().enumerate() {
        for (declaration, binding) in input.symbols.functions().iter().copied().enumerate() {
            let syntax = binding.declaration();
            ledger.charge(1, syntax.span()).map_err(|cause| {
                ScriptPackageError::at(Some(module), Some(declaration as u64), cause)
            })?;
            let function_index = if binding.inclusion() == LegacyFunctionInclusion::Active {
                let name = binding.name();
                let bytes = name.as_bytes().len() as u64;
                ledger
                    .charge(bytes, name.lexeme().span())
                    .map_err(|cause| {
                        ScriptPackageError::at(Some(module), Some(declaration as u64), cause)
                    })?;
                let function = scope.names.len();
                if scope
                    .names
                    .insert(
                        FunctionCallName { module, name },
                        FunctionCallTarget {
                            origin: FunctionCallOrigin::Direct,
                            function,
                            visibility: binding.visibility(),
                        },
                    )
                    .is_some()
                {
                    return Err(error(
                        Some(module),
                        Some(declaration),
                        FunctionCompileErrorKind::InvalidStructure,
                        ledger,
                    ));
                }
                scope.largest_name_bytes = scope.largest_name_bytes.max(bytes);
                Some(function as u64)
            } else {
                None
            };
            declarations.push(CompiledModuleDeclaration {
                span: syntax.span(),
                parameters: syntax.parameters().len() as u64,
                visibility: binding.visibility(),
                inclusion: binding.inclusion(),
                function_index,
            });
        }
    }
    if declarations.len() as u64 != ledger.usage.declarations
        || scope.names.len() as u64 != ledger.usage.functions
    {
        return Err(error(
            None,
            None,
            FunctionCompileErrorKind::InvalidStructure,
            ledger,
        ));
    }
    Ok(PackageInventory {
        scope,
        declarations,
    })
}

fn error(
    module: Option<usize>,
    declaration: Option<usize>,
    kind: FunctionCompileErrorKind,
    ledger: &ModuleCompilationLedger,
) -> ScriptPackageError {
    ScriptPackageError::at(
        module,
        declaration.map(|index| index as u64),
        ledger.locate(function_compile_error::at(0..0, kind)),
    )
}
