// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::FunctionCompileErrorKind;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_program;
use crate::compilation::bound_module::package::{
    CompiledScriptModule, CompiledScriptPackage, ScriptModuleDefinition, ScriptPackageBudget,
    ScriptPackageError, ScriptPackageUsage,
};
use crate::compilation::bound_module::package::{
    package_catalogue, package_inventory, package_lowering, package_sort, package_totals,
};
use crate::compilation::bound_module::{CompiledModule, ModuleCompileError};
use crate::compilation::function_compile_error;

/// Compiles a closed supplied module catalog into one owned verified primitive image.
///
/// Immediate file-qualified/local calls and function references share global wide
/// targets and runtime budgets. Unqualified lookup stays local; ordinary code cannot
/// reference developer-only definitions. All declarations retain input/source order.
/// Names/code/maps survive source/tree/symbol destruction. No filesystem access,
/// include expansion, methods/threads, host services or partial activation occurs.
/// Caller-owned parsing/binding and whole-image kernel proof have separate budgets.
/// # Errors
/// Invalid/duplicate/empty names, unknown/unsupported/developer calls, source errors,
/// global/local work/storage limits, overflow, fallible allocation or proof failures
/// publish no package. Errors locate numeric module/local declaration and source
/// span without names/paths; failed admitted work/peak metadata stay recorded.
pub fn compile_bound_primitive_package(
    inputs: &[ScriptModuleDefinition<'_, '_, '_, '_>],
    budget: ScriptPackageBudget,
) -> Result<CompiledScriptPackage, ScriptPackageError> {
    let totals = package_totals::count(inputs, budget)?;
    let mut ledger = ModuleCompilationLedger::aggregate(totals, budget.compilation)
        .map_err(|cause| ScriptPackageError::at(None, None, cause))?;
    let catalogue = package_catalogue::build(inputs, budget, &mut ledger)?;
    let order = package_sort::build(&catalogue.names, &mut ledger)?;
    let (mut staged, declarations) = {
        let inventory =
            package_inventory::build(inputs, &catalogue.names, &order, budget.path, &mut ledger)?;
        let staged = package_lowering::lower(inputs, &inventory.scope, &mut ledger)?;
        (staged, inventory.declarations)
    };
    let program = module_program::verify(
        &staged.code,
        &mut staged.maps,
        &declarations,
        &mut ledger,
        0..0,
    )
    .map_err(|error| located(error, &catalogue.modules))?;
    let retained_names = package_catalogue::bytes(
        catalogue.modules.len() as u64,
        size_of::<crate::binding::CanonicalScriptPath>() + size_of::<CompiledScriptModule>(),
    )
    .and_then(|rows| {
        rows.checked_add(catalogue.canonical_bytes)
            .ok_or_else(|| function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow))
    })
    .and_then(|bytes| {
        bytes
            .checked_add(ledger.usage.retained_metadata_bytes)
            .ok_or_else(|| function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow))
    })
    .map_err(|cause| ScriptPackageError::at(None, None, ledger.locate(cause)))?;
    let usage = ScriptPackageUsage {
        modules: inputs.len() as u64,
        canonical_module_bytes: catalogue.canonical_bytes,
        compilation: ledger.usage,
        retained_metadata_bytes: retained_names,
    };
    Ok(CompiledScriptPackage {
        image: CompiledModule {
            program,
            declarations,
            functions: staged.maps,
            usage: ledger.usage,
        },
        names: catalogue.names,
        modules: catalogue.modules,
        usage,
    })
}

fn located(error: ModuleCompileError, modules: &[CompiledScriptModule]) -> ScriptPackageError {
    let declaration = error.declaration_index();
    let located = declaration.and_then(|declaration| {
        let after = modules.partition_point(|module| module.declarations.start <= declaration);
        let module = after.checked_sub(1)?;
        let range = &modules[module].declarations;
        range
            .contains(&declaration)
            .then_some((module, declaration - range.start))
    });
    ScriptPackageError::at(
        located.map(|(module, _)| module),
        located.map(|(_, declaration)| declaration),
        error.cause,
    )
}
