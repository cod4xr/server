// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::bound_module::package::{
    ScriptModuleDefinition, ScriptPackageBudget, ScriptPackageError,
};
use crate::compilation::function_compile_error;
use crate::compilation::{
    FunctionCompileErrorKind, ModuleCompilationResource, ModuleCompilationUsage,
};

pub(super) fn count(
    inputs: &[ScriptModuleDefinition<'_, '_, '_, '_>],
    budget: ScriptPackageBudget,
) -> Result<ModuleCompilationUsage, ScriptPackageError> {
    let modules = inputs.len() as u64;
    check(
        None,
        ModuleCompilationResource::Modules,
        modules,
        budget.max_modules,
        0,
    )?;
    // Prepay the complete fixed-cost module inventory before visiting any record.
    check(
        None,
        ModuleCompilationResource::Steps,
        modules,
        budget.compilation.max_steps,
        0,
    )?;
    let mut usage = ModuleCompilationUsage {
        compiler_steps: modules,
        ..ModuleCompilationUsage::default()
    };
    let mut bytes = 0_u64;
    for (index, input) in inputs.iter().enumerate() {
        let add = |left: u64, right: u64| {
            left.checked_add(right)
                .ok_or_else(|| error(Some(index), FunctionCompileErrorKind::SizeOverflow, modules))
        };
        usage.declarations = add(usage.declarations, input.symbols.functions().len() as u64)?;
        usage.functions = add(usage.functions, input.symbols.usage().active_functions)?;
        bytes = add(bytes, input.name.len() as u64)?;
        for (resource, requested, maximum) in [
            (
                ModuleCompilationResource::Declarations,
                usage.declarations,
                budget.compilation.max_declarations,
            ),
            (
                ModuleCompilationResource::Functions,
                usage.functions,
                budget.compilation.max_functions,
            ),
            (
                ModuleCompilationResource::ModuleNameBytes,
                bytes,
                budget.max_path_input_bytes,
            ),
        ] {
            check(Some(index), resource, requested, maximum, modules)?;
        }
    }
    Ok(usage)
}

fn check(
    module: Option<usize>,
    resource: ModuleCompilationResource,
    requested: u64,
    maximum: u64,
    steps: u64,
) -> Result<(), ScriptPackageError> {
    if requested > maximum {
        return Err(error(
            module,
            FunctionCompileErrorKind::ModuleBudgetExceeded {
                resource,
                requested,
                maximum,
            },
            steps,
        ));
    }
    Ok(())
}

fn error(module: Option<usize>, kind: FunctionCompileErrorKind, steps: u64) -> ScriptPackageError {
    let mut cause = function_compile_error::at(0..0, kind);
    cause.work_steps = steps;
    ScriptPackageError::at(module, None, cause)
}
