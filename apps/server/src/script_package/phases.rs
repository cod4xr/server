// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols};
use cod4x_script::declaration::LegacyModuleSyntax;
use cod4x_script::source::SourceText;
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxTree};

use super::frontend::{self, ScriptPackageFrontend};
use crate::app_error::AppError;

pub(crate) fn index<'source>(
    inputs: &'source [Vec<u8>],
    ledger: &mut ScriptPackageFrontend,
) -> Result<Vec<SourceText<'source>>, AppError> {
    let mut sources = frontend::rows(inputs.len())?;
    for (module, bytes) in inputs.iter().enumerate() {
        let source = SourceText::index(bytes, ledger.source).map_err(|error| {
            tracing::error!(event = "script.failed", phase = "index", module, kind = ?error,
                "script package source admission failed");
            AppError::ScriptSource(error)
        })?;
        ledger.indexed(&source)?;
        sources.push(source);
    }
    Ok(sources)
}

pub(crate) fn parse<'source>(
    sources: &[SourceText<'source>],
    ledger: &mut ScriptPackageFrontend,
) -> Result<Vec<LegacySyntaxTree<'source>>, AppError> {
    let mut trees = frontend::rows(sources.len())?;
    for (module, source) in sources.iter().enumerate() {
        let tree = LegacySyntaxTree::parse_structure(source, LegacySyntaxMode::Script, ledger.syntax)
            .map_err(|error| {
                let position = source.location(error.offset).ok();
                tracing::error!(event = "script.failed", phase = "parse", module, kind = ?error.kind,
                    offset = error.offset, line = position.map(|value| value.line),
                    byte_column = position.map(|value| value.byte_column), "script package syntax failed");
                AppError::ScriptSyntax(error)
            })?;
        ledger.parsed(&tree)?;
        trees.push(tree);
    }
    Ok(trees)
}

pub(crate) fn declare<'tree, 'source>(
    sources: &[SourceText<'source>],
    trees: &'tree [LegacySyntaxTree<'source>],
    ledger: &mut ScriptPackageFrontend,
) -> Result<Vec<LegacyModuleSyntax<'tree, 'source>>, AppError> {
    if sources.len() != trees.len() {
        return Err(AppError::ScriptFeature(
            "source/tree inventory invariant failed",
        ));
    }
    let mut declarations = frontend::rows(trees.len())?;
    for (module, (tree, source)) in trees.iter().zip(sources).enumerate() {
        let syntax = LegacyModuleSyntax::lower(tree, ledger.declaration).map_err(|error| {
            let position = source.location(error.offset).ok();
            tracing::error!(event = "script.failed", phase = "declarations", module, kind = ?error.kind,
                offset = error.offset, line = position.map(|value| value.line),
                byte_column = position.map(|value| value.byte_column), "script package declarations failed");
            AppError::ScriptDeclaration(error)
        })?;
        ledger.declared(&syntax)?;
        crate::script_execution::check_directives(
            &syntax,
            Some(module as u64),
            crate::script_execution::ScriptDirectiveProfile::ClosedPackage,
        )?;
        declarations.push(syntax);
    }
    Ok(declarations)
}

pub(crate) fn bind<'module, 'tree, 'source>(
    sources: &[SourceText<'source>],
    modules: &'module [LegacyModuleSyntax<'tree, 'source>],
    mode: LegacyDeveloperMode,
    ledger: &mut ScriptPackageFrontend,
) -> Result<Vec<LegacyModuleSymbols<'module, 'tree, 'source>>, AppError> {
    if sources.len() != modules.len() {
        return Err(AppError::ScriptFeature(
            "source/declaration inventory invariant failed",
        ));
    }
    let mut symbols = frontend::rows(modules.len())?;
    for (module, (syntax, source)) in modules.iter().zip(sources).enumerate() {
        let bindings = LegacyModuleSymbols::bind(syntax, mode, ledger.symbol).map_err(|error| {
            let position = source.location(error.offset).ok();
            let previous = error.previous_offset.and_then(|offset| source.location(offset).ok());
            tracing::error!(event = "script.failed", phase = "binding", module, kind = ?error.kind,
                offset = error.offset, line = position.map(|value| value.line),
                byte_column = position.map(|value| value.byte_column), previous_offset = error.previous_offset,
                previous_line = previous.map(|value| value.line), previous_byte_column = previous.map(|value| value.byte_column),
                "script package local binding failed");
            AppError::ScriptBinding(error)
        })?;
        ledger.bound(&bindings)?;
        symbols.push(bindings);
    }
    Ok(symbols)
}
