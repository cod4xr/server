// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::binding::{LegacyFunctionInclusion, LegacyModuleSymbols};
use cod4x_script::compilation::{self, ModuleCompilationBudget};
use cod4x_script::declaration::{LegacyModuleItem, LegacyModuleSyntax};
use cod4x_script::execution::KernelCallBudget;
use cod4x_script::source::SourceText;

use crate::app_error::AppError;
use crate::script_result;

pub(crate) fn run(
    source: &SourceText<'_>,
    module: &LegacyModuleSyntax<'_, '_>,
    symbols: &LegacyModuleSymbols<'_, '_, '_>,
) -> Result<(), AppError> {
    check_directives(module)?;
    let compiled = compilation::compile_bound_primitive_module(symbols, ModuleCompilationBudget::default()).map_err(|error| {
        let location = source.location(error.cause.span.start).ok();
        tracing::error!(event = "script.failed", phase = "compile", kind = "module", cause = ?error.cause.kind,
            declaration = error.declaration_index(), offset = error.cause.span.start,
            line = location.map(|value| value.line), byte_column = location.map(|value| value.byte_column),
            compiler_steps = error.cause.work_steps, peak_metadata_bytes = error.cause.peak_metadata_bytes,
            "primitive GSC compilation failed");
        AppError::ScriptCompilation(error)
    })?;
    let declaration = symbols
        .functions()
        .iter()
        .position(|function| {
            function.inclusion() == LegacyFunctionInclusion::Active
                && function.name().as_bytes().eq_ignore_ascii_case(b"main")
        })
        .ok_or_else(|| feature("entry", "an active local main function is required"))?;
    let entry = compiled
        .function_for_declaration(declaration as u64)
        .ok_or_else(|| feature("entry", "compiled main entry mapping is invalid"))?;
    let usage = compiled.usage();
    tracing::info!(
        event = "script.compiled",
        profile = "bound_primitive",
        declarations = usage.declarations,
        functions = usage.functions,
        instructions = usage.instructions,
        compiler_steps = usage.compiler_steps,
        peak_metadata_bytes = usage.peak_metadata_bytes,
        verification_steps = usage.verification.verification_steps,
        "primitive GSC module compiled"
    );
    let outcome = compiled.program().execute(entry, &[], KernelCallBudget::default()).map_err(|error| {
        let span = error.function.zip(error.cause.instruction).and_then(|(function, instruction)| compiled.instruction_span(function, instruction));
        let location = span.as_ref().and_then(|span| source.location(span.start).ok());
        tracing::error!(event = "script.failed", phase = "execute", kind = "runtime", cause = ?error.cause.kind,
            function = error.function, instruction = error.cause.instruction, offset = span.map(|span| span.start),
            line = location.map(|value| value.line), byte_column = location.map(|value| value.byte_column),
            steps = error.cause.work_steps, setup_work = error.setup_work, peak_frames = error.peak_frames,
            "primitive GSC execution failed");
        AppError::ScriptExecution(error)
    })?;
    script_result::write_stdout(outcome.value).map_err(|error| {
        tracing::error!(event = "script.failed", phase = "output", kind = ?error.kind(), "script result output failed");
        AppError::ScriptOutput(error)
    })?;
    let usage = outcome.usage;
    tracing::info!(
        event = "script.executed",
        profile = "bound_primitive",
        result_kind = script_result::kind(outcome.value),
        steps = usage.steps,
        setup_work = usage.setup_work,
        peak_frames = usage.peak_frames,
        peak_frame_bytes = usage.peak_frame_bytes,
        peak_stack_values = usage.peak_stack_values,
        peak_stack_bytes = usage.peak_stack_bytes,
        peak_local_values = usage.peak_local_values,
        peak_local_bytes = usage.peak_local_bytes,
        "primitive GSC main completed"
    );
    Ok(())
}

fn check_directives(module: &LegacyModuleSyntax<'_, '_>) -> Result<(), AppError> {
    if !module.includes().is_empty() {
        return Err(feature(
            "directives",
            "include resolution is not implemented",
        ));
    }
    if module
        .items()
        .iter()
        .any(|item| matches!(item, LegacyModuleItem::UsingAnimTree(_)))
    {
        return Err(feature(
            "directives",
            "animation-tree resolution is not implemented",
        ));
    }
    Ok(())
}

fn feature(phase: &'static str, message: &'static str) -> AppError {
    tracing::error!(
        event = "script.failed",
        phase,
        kind = "unsupported",
        "{message}"
    );
    AppError::ScriptFeature(message)
}
