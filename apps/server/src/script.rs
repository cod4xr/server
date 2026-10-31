// SPDX-License-Identifier: AGPL-3.0-or-later
use std::path::Path;

use cod4x_script::declaration::{DeclarationBudget, LegacyModuleItem, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

use crate::app_error::AppError;
use crate::content_snapshot;

pub(crate) fn inspect(path: &Path) -> Result<(), AppError> {
    tracing::info!(
        event = "script.started",
        mode = "legacy_structure",
        "inspecting legacy GSC structure"
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
    let declarations = module.usage();
    let usage = tree.usage();
    tracing::info!(
        event = "script.syntax_inspected",
        profile = "legacy",
        source_bytes = bytes.len(),
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
        functions = module
            .items()
            .iter()
            .filter(|item| matches!(item, LegacyModuleItem::Function(_)))
            .count(),
        parameters = declarations.parameters,
        declaration_steps = declarations.steps,
        declaration_metadata_bytes = declarations.metadata_bytes,
        "legacy GSC structure inspected; compilation and execution remain unavailable"
    );
    Ok(())
}
