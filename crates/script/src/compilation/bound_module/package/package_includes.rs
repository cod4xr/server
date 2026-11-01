// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashSet;
use std::ops::Range;

use crate::binding::CanonicalScriptPath;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::package::package_catalogue;
use crate::compilation::bound_module::package::{
    ScriptModuleDefinition, ScriptPackageBudget, ScriptPackageError,
};
use crate::compilation::function_compile_error;
use crate::compilation::module_call_scope::ModuleCallScope;
use crate::compilation::{FunctionCompileErrorKind, ModuleCompilationResource};

pub(super) struct ResolvedInclude {
    pub(super) module: usize,
    pub(super) target: usize,
    pub(super) span: Range<u64>,
}

pub(super) struct ResolvedIncludes {
    pub(super) rows: Vec<ResolvedInclude>,
    pub(super) aliases: u64,
    pub(super) bytes: u64,
}

pub(super) fn resolve(
    inputs: &[ScriptModuleDefinition<'_, '_, '_, '_>],
    names: &[CanonicalScriptPath],
    order: &[usize],
    count: u64,
    budget: ScriptPackageBudget,
    ledger: &mut ModuleCompilationLedger,
) -> Result<ResolvedIncludes, ScriptPackageError> {
    let bytes = package_catalogue::bytes(count, size_of::<ResolvedInclude>())
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    ledger
        .temporary(bytes, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    let rows = module_storage::reserve(count, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    let mut resolved = ResolvedIncludes {
        rows,
        aliases: 0,
        bytes,
    };
    if count == 0 {
        return Ok(resolved);
    }
    ledger
        .charge(inputs.len() as u64, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    let namespace = ModuleCallScope {
        names,
        order,
        path_budget: budget.path,
    };
    for (module, input) in inputs.iter().enumerate() {
        let entries = input.includes.len();
        if entries == 0 {
            continue;
        }
        let mut seen = duplicate_set(entries, bytes, module, ledger)?;
        for path in input.includes.iter().copied() {
            let span = path.span();
            let target = namespace
                .resolve_with(path, |visits, span| ledger.charge(visits, span))
                .map_err(|cause| package_catalogue::locate(Some(module), cause, ledger))?;
            if !seen.insert(target) {
                return Err(fail(
                    module,
                    span,
                    FunctionCompileErrorKind::DuplicateInclude,
                    ledger,
                ));
            }
            let definitions = inputs.get(target).ok_or_else(|| {
                fail(
                    module,
                    span.clone(),
                    FunctionCompileErrorKind::InvalidStructure,
                    ledger,
                )
            })?;
            resolved.aliases = resolved
                .aliases
                .checked_add(definitions.symbols.usage().active_functions)
                .ok_or_else(|| {
                    fail(
                        module,
                        span.clone(),
                        FunctionCompileErrorKind::SizeOverflow,
                        ledger,
                    )
                })?;
            ledger
                .check(
                    ModuleCompilationResource::ImportedFunctions,
                    resolved.aliases,
                    budget.max_imported_functions,
                    span.clone(),
                )
                .map_err(|cause| package_catalogue::locate(Some(module), cause, ledger))?;
            resolved.rows.push(ResolvedInclude {
                module,
                target,
                span,
            });
        }
    }
    if resolved.rows.len() as u64 != count {
        return Err(package_catalogue::locate(
            None,
            function_compile_error::at(0..0, FunctionCompileErrorKind::InvalidStructure),
            ledger,
        ));
    }
    Ok(resolved)
}

fn duplicate_set(
    entries: usize,
    bytes: u64,
    module: usize,
    ledger: &mut ModuleCompilationLedger,
) -> Result<HashSet<usize>, ScriptPackageError> {
    let scratch = package_catalogue::bytes(entries as u64, size_of::<usize>())
        .and_then(|scratch| {
            scratch.checked_add(bytes).ok_or_else(|| {
                function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow)
            })
        })
        .map_err(|cause| package_catalogue::locate(Some(module), cause, ledger))?;
    ledger
        .temporary(scratch, 0..0)
        .map_err(|cause| package_catalogue::locate(Some(module), cause, ledger))?;
    let visits = (entries as u64)
        .checked_mul(2)
        .ok_or_else(|| fail(module, 0..0, FunctionCompileErrorKind::SizeOverflow, ledger))?;
    ledger
        .charge(visits, 0..0)
        .map_err(|cause| package_catalogue::locate(Some(module), cause, ledger))?;
    let mut seen = HashSet::new();
    seen.try_reserve(entries).map_err(|_| {
        fail(
            module,
            0..0,
            FunctionCompileErrorKind::AllocationFailed,
            ledger,
        )
    })?;
    Ok(seen)
}

pub(super) fn fail(
    module: usize,
    span: Range<u64>,
    kind: FunctionCompileErrorKind,
    ledger: &ModuleCompilationLedger,
) -> ScriptPackageError {
    package_catalogue::locate(Some(module), function_compile_error::at(span, kind), ledger)
}
