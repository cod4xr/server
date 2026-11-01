// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::binding::{LegacyFunctionInclusion, LegacyModuleSymbols};
use cod4x_script::compilation::{self, ModuleCompilationBudget};
use cod4x_script::declaration::{LegacyModuleItem, LegacyModuleSyntax};
use cod4x_script::source::SourceText;

use crate::app_error::AppError;
use crate::script_image::ScriptImage;
use crate::script_runtime;

#[derive(Clone, Copy)]
pub(crate) enum ScriptDirectiveProfile {
    SingleModule,
    ClosedPackage,
}

pub(crate) fn run(
    source: &SourceText<'_>,
    module: &LegacyModuleSyntax<'_, '_>,
    symbols: &LegacyModuleSymbols<'_, '_, '_>,
) -> Result<(), AppError> {
    check_directives(module, None, ScriptDirectiveProfile::SingleModule)?;
    let compiled = compilation::compile_bound_primitive_module(symbols, ModuleCompilationBudget::default()).map_err(|error| {
        let location = source.location(error.cause.span.start).ok();
        tracing::error!(event = "script.failed", phase = "compile", kind = "module", cause = ?error.cause.kind,
            declaration = error.declaration_index(), offset = error.cause.span.start,
            line = location.map(|value| value.line), byte_column = location.map(|value| value.byte_column),
            compiler_steps = error.cause.work_steps, peak_metadata_bytes = error.cause.peak_metadata_bytes,
            "primitive GSC compilation failed");
        AppError::ScriptCompilation(error)
    })?;
    let declaration = entry(symbols, None)?;
    let entry = compiled
        .function_for_declaration(declaration)
        .ok_or_else(|| feature("entry", "compiled main entry mapping is invalid"))?;
    let usage = compiled.usage();
    tracing::info!(
        event = "script.compiled",
        profile = "bound_primitive",
        declarations = usage.declarations,
        functions = usage.functions,
        instructions = usage.instructions,
        string_literals = usage.string_literals,
        staged_string_bytes = usage.staged_string_bytes,
        field_names = usage.field_names,
        staged_field_bytes = usage.staged_field_bytes,
        compiler_steps = usage.compiler_steps,
        peak_metadata_bytes = usage.peak_metadata_bytes,
        verification_steps = usage.verification.verification_steps,
        "primitive GSC module compiled"
    );
    script_runtime::run(
        ScriptImage::Module {
            source,
            compiled: &compiled,
        },
        entry,
    )
}

pub(crate) fn check_directives(
    module: &LegacyModuleSyntax<'_, '_>,
    index: Option<u64>,
    profile: ScriptDirectiveProfile,
) -> Result<(), AppError> {
    if matches!(profile, ScriptDirectiveProfile::SingleModule) && !module.includes().is_empty() {
        return Err(feature_at(
            index,
            "directives",
            "include resolution is not implemented",
        ));
    }
    if module
        .items()
        .iter()
        .any(|item| matches!(item, LegacyModuleItem::UsingAnimTree(_)))
    {
        return Err(feature_at(
            index,
            "directives",
            "animation-tree resolution is not implemented",
        ));
    }
    Ok(())
}

pub(crate) fn entry(
    symbols: &LegacyModuleSymbols<'_, '_, '_>,
    module: Option<u64>,
) -> Result<u64, AppError> {
    symbols
        .functions()
        .iter()
        .position(|function| {
            function.inclusion() == LegacyFunctionInclusion::Active
                && function.name().as_bytes().eq_ignore_ascii_case(b"main")
        })
        .map(|index| index as u64)
        .ok_or_else(|| feature_at(module, "entry", "an active local main function is required"))
}

fn feature(phase: &'static str, message: &'static str) -> AppError {
    feature_at(None, phase, message)
}

fn feature_at(module: Option<u64>, phase: &'static str, message: &'static str) -> AppError {
    tracing::error!(
        event = "script.failed",
        phase,
        module,
        kind = "unsupported",
        "{message}"
    );
    AppError::ScriptFeature(message)
}
