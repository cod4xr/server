// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::LegacyFunctionInclusion;
use crate::compilation::FunctionCompileErrorKind;
use crate::compilation::bound_module::CompiledModuleFunction;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::package::{ScriptModuleDefinition, ScriptPackageError};
use crate::compilation::bound_module::staged_function::{self, StagedFunctionCode};
use crate::compilation::function_call_scope::FunctionCallScope;
use crate::compilation::function_compile_error;
use crate::compilation::function_lowering_mode::FunctionLoweringMode;

pub(super) struct PackageCode {
    pub(super) code: Vec<StagedFunctionCode>,
    pub(super) maps: Vec<CompiledModuleFunction>,
}

pub(super) fn lower<'tree, 'source>(
    inputs: &[ScriptModuleDefinition<'_, '_, 'tree, 'source>],
    scope: &'tree FunctionCallScope<'source>,
    ledger: &mut ModuleCompilationLedger,
) -> Result<PackageCode, ScriptPackageError> {
    let mut code = module_storage::reserve(ledger.usage.functions, 0..0)
        .map_err(|cause| ScriptPackageError::at(None, None, ledger.locate(cause)))?;
    let mut maps = module_storage::reserve(ledger.usage.functions, 0..0)
        .map_err(|cause| ScriptPackageError::at(None, None, ledger.locate(cause)))?;
    let mut global = 0_usize;
    for (module, input) in inputs.iter().enumerate() {
        for (index, binding) in input.symbols.functions().iter().copied().enumerate() {
            let declaration = global;
            global = global.checked_add(1).ok_or_else(|| {
                ScriptPackageError::at(
                    Some(module),
                    Some(index as u64),
                    ledger.locate(function_compile_error::at(
                        binding.declaration().span(),
                        FunctionCompileErrorKind::SizeOverflow,
                    )),
                )
            })?;
            ledger
                .charge(1, binding.declaration().span())
                .map_err(|cause| ScriptPackageError::at(Some(module), Some(index as u64), cause))?;
            if binding.inclusion() != LegacyFunctionInclusion::Active {
                continue;
            }
            let limits = ledger
                .function_budget(binding.declaration().span())
                .map_err(|cause| {
                    ScriptPackageError::at(Some(module), Some(index as u64), ledger.locate(cause))
                })?;
            let mode = FunctionLoweringMode::BoundModule {
                scope,
                caller: binding.visibility(),
                module,
            };
            let (staged, map) =
                staged_function::lower(binding.declaration(), declaration, mode, limits).map_err(
                    |cause| {
                        ScriptPackageError::at(
                            Some(module),
                            Some(index as u64),
                            ledger.function_error(cause),
                        )
                    },
                )?;
            ledger
                .record(map.usage, binding.declaration().span())
                .map_err(|cause| {
                    ScriptPackageError::at(Some(module), Some(index as u64), ledger.locate(cause))
                })?;
            code.push(staged);
            maps.push(map);
        }
    }
    Ok(PackageCode { code, maps })
}
