// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{CanonicalScriptPath, LegacyScriptPath};
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::package::{
    CompiledScriptModule, ScriptModuleDefinition, ScriptPackageBudget, ScriptPackageError,
};
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

pub(super) struct PackageCatalogue {
    pub(super) names: Vec<CanonicalScriptPath>,
    pub(super) modules: Vec<CompiledScriptModule>,
    pub(super) canonical_bytes: u64,
}

pub(super) fn build(
    inputs: &[ScriptModuleDefinition<'_, '_, '_, '_>],
    budget: ScriptPackageBudget,
    ledger: &mut ModuleCompilationLedger,
) -> Result<PackageCatalogue, ScriptPackageError> {
    let count = inputs.len() as u64;
    let rows = bytes(
        count,
        size_of::<CanonicalScriptPath>() + size_of::<CompiledScriptModule>() + size_of::<usize>(),
    )
    .map_err(|cause| locate(None, cause, ledger))?;
    ledger
        .extend_base(rows, 0..0)
        .map_err(|cause| locate(None, cause, ledger))?;
    let scratch = bytes(count, size_of::<LegacyScriptPath<'_>>())
        .map_err(|cause| locate(None, cause, ledger))?;
    ledger
        .temporary(scratch, 0..0)
        .map_err(|cause| locate(None, cause, ledger))?;
    let mut views =
        module_storage::reserve(count, 0..0).map_err(|cause| locate(None, cause, ledger))?;
    let mut canonical_bytes = 0_u64;
    for (index, input) in inputs.iter().enumerate() {
        let visits = bytes(input.name.len() as u64, 2)
            .map_err(|cause| locate(Some(index), cause, ledger))?;
        ledger
            .charge(visits, 0..0)
            .map_err(|cause| locate(Some(index), cause, ledger))?;
        let view = LegacyScriptPath::new(input.name, budget.path)
            .map_err(|cause| {
                locate(
                    Some(index),
                    function_compile_error::at(
                        0..0,
                        FunctionCompileErrorKind::ScriptPath(cause.kind),
                    ),
                    ledger,
                )
            })?
            .value;
        if view.canonical_len() == 0 {
            return Err(locate(
                Some(index),
                function_compile_error::at(0..0, FunctionCompileErrorKind::EmptyModuleName),
                ledger,
            ));
        }
        canonical_bytes = canonical_bytes
            .checked_add(view.canonical_len())
            .ok_or_else(|| {
                locate(
                    Some(index),
                    function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow),
                    ledger,
                )
            })?;
        views.push(view);
    }
    own_catalogue(inputs, budget, ledger, views, canonical_bytes, scratch)
}

fn own_catalogue<'source>(
    inputs: &[ScriptModuleDefinition<'_, '_, '_, 'source>],
    budget: ScriptPackageBudget,
    ledger: &mut ModuleCompilationLedger,
    views: Vec<LegacyScriptPath<'source>>,
    canonical_bytes: u64,
    scratch: u64,
) -> Result<PackageCatalogue, ScriptPackageError> {
    let count = inputs.len() as u64;
    ledger
        .extend_base(canonical_bytes, 0..0)
        .map_err(|cause| locate(None, cause, ledger))?;
    ledger
        .temporary(scratch, 0..0)
        .map_err(|cause| locate(None, cause, ledger))?;
    let mut names =
        module_storage::reserve(count, 0..0).map_err(|cause| locate(None, cause, ledger))?;
    let mut modules =
        module_storage::reserve(count, 0..0).map_err(|cause| locate(None, cause, ledger))?;
    let (mut declarations, mut functions) = (0_u64, 0_u64);
    for (index, (input, view)) in inputs.iter().zip(views).enumerate() {
        let visits = (input.name.len() as u64)
            .checked_add(view.canonical_len())
            .ok_or_else(|| {
                locate(
                    Some(index),
                    function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow),
                    ledger,
                )
            })?;
        ledger
            .charge(visits, 0..0)
            .map_err(|cause| locate(Some(index), cause, ledger))?;
        let name = view
            .copy_owned(budget.path)
            .map_err(|cause| {
                locate(
                    Some(index),
                    function_compile_error::at(
                        0..0,
                        FunctionCompileErrorKind::ScriptPath(cause.kind),
                    ),
                    ledger,
                )
            })?
            .value;
        let declaration_end = declarations
            .checked_add(input.symbols.functions().len() as u64)
            .ok_or_else(|| {
                locate(
                    Some(index),
                    function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow),
                    ledger,
                )
            })?;
        let function_end = functions
            .checked_add(input.symbols.usage().active_functions)
            .ok_or_else(|| {
                locate(
                    Some(index),
                    function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow),
                    ledger,
                )
            })?;
        modules.push(CompiledScriptModule {
            declarations: declarations..declaration_end,
            functions: functions..function_end,
            developer_mode: input.symbols.mode(),
        });
        names.push(name);
        declarations = declaration_end;
        functions = function_end;
    }
    if names.len() != inputs.len()
        || modules.len() != inputs.len()
        || declarations != ledger.usage.declarations
        || functions != ledger.usage.functions
    {
        return Err(locate(
            None,
            function_compile_error::at(0..0, FunctionCompileErrorKind::InvalidStructure),
            ledger,
        ));
    }
    Ok(PackageCatalogue {
        names,
        modules,
        canonical_bytes,
    })
}

pub(super) fn bytes(count: u64, size: usize) -> Result<u64, FunctionCompileError> {
    count
        .checked_mul(size as u64)
        .ok_or_else(|| function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow))
}

pub(super) fn locate(
    module: Option<usize>,
    cause: FunctionCompileError,
    ledger: &ModuleCompilationLedger,
) -> ScriptPackageError {
    ScriptPackageError::at(module, None, ledger.locate(cause))
}
