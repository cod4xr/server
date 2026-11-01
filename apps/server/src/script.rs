// SPDX-License-Identifier: AGPL-3.0-or-later
use std::path::Path;

use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

use crate::app_error::AppError;
use crate::content_snapshot;
use crate::script_execution;

#[derive(Clone, Copy)]
enum Operation {
    Inspect,
    Execute,
}

pub(crate) fn inspect(path: &Path, developer_mode: LegacyDeveloperMode) -> Result<(), AppError> {
    process(path, developer_mode, Operation::Inspect)
}

pub(crate) fn execute(path: &Path, developer_mode: LegacyDeveloperMode) -> Result<(), AppError> {
    process(path, developer_mode, Operation::Execute)
}

fn process(
    path: &Path,
    developer_mode: LegacyDeveloperMode,
    operation: Operation,
) -> Result<(), AppError> {
    tracing::info!(
        event = "script.started",
        mode = match operation {
            Operation::Inspect => "legacy_structure",
            Operation::Execute => "primitive_execution",
        },
        "loading legacy GSC source"
    );
    let budget = SourceBudget::default();
    let bytes = content_snapshot::read(path, budget.max_source_bytes).map_err(|error| {
        tracing::error!(
            event = "script.failed",
            phase = "read",
            kind = error.category(),
            "script snapshot could not be read"
        );
        AppError::Snapshot(error)
    })?;
    let source = SourceText::index(&bytes, budget).map_err(|error| {
        tracing::error!(event = "script.failed", phase = "index", kind = ?error, "script source admission failed");
        AppError::ScriptSource(error)
    })?;
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .map_err(|error| {
        let location = source.location(error.offset).ok();
        tracing::error!(event = "script.failed", phase = "parse", kind = ?error.kind,
            offset = error.offset, line = location.map(|location| location.line),
            byte_column = location.map(|location| location.byte_column),
            "legacy GSC structural syntax failed");
        AppError::ScriptSyntax(error)
    })?;
    let module =
        LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).map_err(|error| {
            let location = source.location(error.offset).ok();
            tracing::error!(event = "script.failed", phase = "declarations", kind = ?error.kind,
            offset = error.offset, line = location.map(|location| location.line),
            byte_column = location.map(|location| location.byte_column),
            "legacy GSC declaration extraction failed");
            AppError::ScriptDeclaration(error)
        })?;
    let symbols = bind_module(&source, &module, developer_mode)?;
    match operation {
        Operation::Inspect => {
            report(&source, &tree, &module, &symbols);
            Ok(())
        }
        Operation::Execute => script_execution::run(&source, &module, &symbols),
    }
}

fn report(
    source: &SourceText<'_>,
    tree: &LegacySyntaxTree<'_>,
    module: &LegacyModuleSyntax<'_, '_>,
    symbols: &LegacyModuleSymbols<'_, '_, '_>,
) {
    let declarations = module.usage();
    let usage = tree.usage();
    let bindings = symbols.usage();
    tracing::info!(
        event = "script.syntax_inspected",
        profile = "legacy",
        source_bytes = source.bytes().len(),
        lines = source.line_count(),
        source_tokens = usage.source_tokens,
        nodes = usage.nodes,
        child_edges = usage.child_edges,
        reductions = usage.reductions,
        grammar_steps = usage.steps,
        scan_steps = usage.lexical.steps,
        peak_stack_entries = usage.peak_stack_entries,
        peak_metadata_bytes = usage.peak_metadata_bytes,
        source_index_bytes = source.metadata_bytes(),
        items = declarations.items,
        includes = declarations.includes,
        functions = bindings.functions,
        parameters = declarations.parameters,
        declaration_steps = declarations.steps,
        declaration_metadata_bytes = declarations.metadata_bytes,
        developer_scripts = ?symbols.mode(), active_functions = bindings.active_functions,
        ordinary_functions = bindings.ordinary_functions, developer_functions = bindings.developer_functions,
        binding_steps = bindings.steps, binding_metadata_bytes = bindings.metadata_bytes,
        function_name_bytes = bindings.name_bytes,
        "legacy GSC structure inspected; this mode does not compile or execute"
    );
}

fn bind_module<'module, 'tree, 'source>(
    source: &SourceText<'source>,
    module: &'module LegacyModuleSyntax<'tree, 'source>,
    mode: LegacyDeveloperMode,
) -> Result<LegacyModuleSymbols<'module, 'tree, 'source>, AppError> {
    LegacyModuleSymbols::bind(module, mode, SymbolBudget::default()).map_err(|error| {
        let location = source.location(error.offset).ok();
        let previous = error.previous_offset.and_then(|offset| source.location(offset).ok());
        tracing::error!(event = "script.failed", phase = "binding", kind = ?error.kind,
            offset = error.offset, line = location.map(|location| location.line),
            byte_column = location.map(|location| location.byte_column),
            previous_offset = error.previous_offset, previous_line = previous.map(|location| location.line),
            previous_byte_column = previous.map(|location| location.byte_column),
            "legacy GSC local function binding failed");
        AppError::ScriptBinding(error)
    })
}
