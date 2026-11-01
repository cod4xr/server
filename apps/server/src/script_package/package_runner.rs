// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::binding::{CanonicalScriptPath, LegacyDeveloperMode};
use cod4x_script::compilation::{self, ScriptModuleDefinition, ScriptPackageBudget};
use cod4x_script::source::SourceBudget;

use super::frontend::{self, ScriptPackageFrontend};
use super::options::ScriptPackageInput;
use super::phases;
use crate::app_error::AppError;
use crate::content_snapshot;
use crate::script_execution;
use crate::script_image::ScriptImage;
use crate::script_runtime;

pub(crate) fn run(
    entry: &CanonicalScriptPath,
    inputs: &[ScriptPackageInput],
    mode: LegacyDeveloperMode,
) -> Result<(), AppError> {
    tracing::info!(
        event = "script.started",
        mode = "primitive_package_execution",
        modules = inputs.len(),
        "loading supplied GSC package sources"
    );
    let bytes = snapshots(inputs)?;
    let mut ledger = ScriptPackageFrontend::default();
    let sources = phases::index(&bytes, &mut ledger)?;
    let trees = phases::parse(&sources, &mut ledger)?;
    let modules = phases::declare(&sources, &trees, &mut ledger)?;
    let symbols = phases::bind(&sources, &modules, mode, &mut ledger)?;
    if inputs.len() != symbols.len() {
        return Err(AppError::ScriptFeature(
            "package symbol inventory invariant failed",
        ));
    }
    let mut definitions = frontend::rows(inputs.len())?;
    for (input, symbol) in inputs.iter().zip(&symbols) {
        definitions.push(ScriptModuleDefinition::new(input.name.as_bytes(), symbol));
    }
    let compiled = compilation::compile_bound_primitive_package(&definitions, ScriptPackageBudget::default())
        .map_err(|error| {
            let position = error.module_index().and_then(|index| usize::try_from(index).ok())
                .and_then(|index| sources.get(index)).and_then(|source| source.location(error.cause.span.start).ok());
            tracing::error!(event = "script.failed", phase = "compile", kind = "package", cause = ?error.cause.kind,
                module = error.module_index(), declaration = error.declaration_index(), offset = error.cause.span.start,
                line = position.map(|value| value.line), byte_column = position.map(|value| value.byte_column),
                compiler_steps = error.cause.work_steps, peak_metadata_bytes = error.cause.peak_metadata_bytes,
                "primitive GSC package compilation failed");
            AppError::ScriptPackage(error)
        })?;
    // Complete canonical names were admitted/copied once by the CLI. Comparing the
    // owned byte slices visits at most the admitted aggregate catalog-name bytes.
    let module = inputs
        .iter()
        .position(|input| input.name.as_bytes() == entry.as_bytes())
        .ok_or_else(|| entry_error("entry module is not supplied"))?;
    let declaration = script_execution::entry(&symbols[module], Some(module as u64))?;
    let function = compiled
        .function_for_declaration(module as u64, declaration)
        .ok_or_else(|| entry_error("compiled package entry mapping is invalid"))?;
    report(&compiled, &ledger);
    // The verified image is independently owned; keep only line indices/source
    // snapshots for runtime locations, not compiler/tree/binding staging.
    drop(definitions);
    drop(symbols);
    drop(modules);
    drop(trees);
    script_runtime::run(
        ScriptImage::Package {
            sources: &sources,
            compiled: &compiled,
        },
        function,
    )
}

fn snapshots(inputs: &[ScriptPackageInput]) -> Result<Vec<Vec<u8>>, AppError> {
    let mut rows = frontend::rows(inputs.len())?;
    let mut remaining = SourceBudget::default().max_source_bytes;
    for (module, input) in inputs.iter().enumerate() {
        let bytes = content_snapshot::read(&input.path, remaining).map_err(|error| {
            tracing::error!(
                event = "script.failed",
                phase = "read",
                module,
                kind = error.category(),
                "script package snapshot could not be read"
            );
            AppError::Snapshot(error)
        })?;
        remaining = remaining
            .checked_sub(bytes.len() as u64)
            .ok_or(AppError::ScriptFeature(
                "source snapshot accounting invariant failed",
            ))?;
        rows.push(bytes);
    }
    Ok(rows)
}

fn entry_error(message: &'static str) -> AppError {
    tracing::error!(
        event = "script.failed",
        phase = "entry",
        kind = "unsupported",
        "{message}"
    );
    AppError::ScriptFeature(message)
}

fn report(compiled: &compilation::CompiledScriptPackage, ledger: &ScriptPackageFrontend) {
    let package = compiled.usage();
    let usage = package.compilation;
    let initial = ScriptPackageFrontend::default();
    tracing::info!(
        event = "script.compiled",
        profile = "bound_primitive_package",
        modules = package.modules,
        source_bytes = initial.source.max_source_bytes - ledger.source.max_source_bytes,
        source_index_bytes = initial.source.max_metadata_bytes - ledger.source.max_metadata_bytes,
        grammar_steps = initial.syntax.max_steps - ledger.syntax.max_steps,
        scan_steps = initial.syntax.lexical.max_steps - ledger.syntax.lexical.max_steps,
        syntax_peak_sum_bytes =
            initial.syntax.max_metadata_bytes - ledger.syntax.max_metadata_bytes,
        declaration_steps = initial.declaration.max_steps - ledger.declaration.max_steps,
        declaration_metadata_bytes =
            initial.declaration.max_metadata_bytes - ledger.declaration.max_metadata_bytes,
        binding_steps = initial.symbol.max_steps - ledger.symbol.max_steps,
        binding_metadata_bytes =
            initial.symbol.max_metadata_bytes - ledger.symbol.max_metadata_bytes,
        canonical_module_bytes = package.canonical_module_bytes,
        declarations = usage.declarations,
        functions = usage.functions,
        instructions = usage.instructions,
        string_literals = usage.string_literals,
        staged_string_bytes = usage.staged_string_bytes,
        field_names = usage.field_names,
        staged_field_bytes = usage.staged_field_bytes,
        compiler_steps = usage.compiler_steps,
        peak_metadata_bytes = usage.peak_metadata_bytes,
        retained_metadata_bytes = package.retained_metadata_bytes,
        verification_steps = usage.verification.verification_steps,
        "primitive GSC package compiled"
    );
}
