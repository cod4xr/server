// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::LegacyFunctionInclusion;
use crate::compilation::FunctionCompileErrorKind;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::package::package_includes::ResolvedIncludes;
use crate::compilation::bound_module::package::{ScriptModuleDefinition, ScriptPackageError};
use crate::compilation::bound_module::package::{package_catalogue, package_includes};
use crate::compilation::function_call_scope::{
    FunctionCallName, FunctionCallOrigin, FunctionCallScope, FunctionCallTarget,
};
use crate::compilation::function_compile_error;

pub(super) fn admit(
    resolved: &ResolvedIncludes,
    ledger: &mut ModuleCompilationLedger,
) -> Result<(), ScriptPackageError> {
    let rows = package_catalogue::bytes(
        resolved.aliases,
        size_of::<(FunctionCallName<'_>, FunctionCallTarget)>(),
    )
    .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    ledger
        .extend_base(rows, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    ledger
        .temporary(resolved.bytes, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))
}

pub(super) fn install<'source>(
    inputs: &[ScriptModuleDefinition<'_, '_, '_, 'source>],
    scope: &mut FunctionCallScope<'source>,
    resolved: &ResolvedIncludes,
    ledger: &mut ModuleCompilationLedger,
) -> Result<(), ScriptPackageError> {
    let mut installed = 0_u64;
    for edge in &resolved.rows {
        for binding in inputs[edge.target].symbols.functions().iter().copied() {
            ledger
                .charge(1, edge.span.clone())
                .map_err(|cause| package_catalogue::locate(Some(edge.module), cause, ledger))?;
            if binding.inclusion() != LegacyFunctionInclusion::Active {
                continue;
            }
            let name = binding.name();
            // Two key hashes and possible matching equality scans: preserve complete
            // identifiers, including the long-name case-identity profile.
            let visits = (name.as_bytes().len() as u64)
                .checked_mul(4)
                .ok_or_else(|| {
                    package_includes::fail(
                        edge.module,
                        edge.span.clone(),
                        FunctionCompileErrorKind::SizeOverflow,
                        ledger,
                    )
                })?;
            ledger
                .charge(visits, edge.span.clone())
                .map_err(|cause| package_catalogue::locate(Some(edge.module), cause, ledger))?;
            let target = scope
                .find(edge.target, name)
                .filter(|target| target.origin == FunctionCallOrigin::Direct)
                .ok_or_else(|| {
                    package_includes::fail(
                        edge.module,
                        edge.span.clone(),
                        FunctionCompileErrorKind::InvalidStructure,
                        ledger,
                    )
                })?;
            if scope
                .names
                .insert(
                    FunctionCallName {
                        module: edge.module,
                        name,
                    },
                    FunctionCallTarget {
                        origin: FunctionCallOrigin::Imported,
                        ..target
                    },
                )
                .is_some()
            {
                return Err(package_includes::fail(
                    edge.module,
                    edge.span.clone(),
                    FunctionCompileErrorKind::IncludeFunctionConflict,
                    ledger,
                ));
            }
            installed = installed.checked_add(1).ok_or_else(|| {
                package_includes::fail(
                    edge.module,
                    edge.span.clone(),
                    FunctionCompileErrorKind::SizeOverflow,
                    ledger,
                )
            })?;
        }
    }
    if installed != resolved.aliases {
        return Err(package_catalogue::locate(
            None,
            function_compile_error::at(0..0, FunctionCompileErrorKind::InvalidStructure),
            ledger,
        ));
    }
    Ok(())
}
