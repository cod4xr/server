// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::binding::{LegacyModuleSymbols, SymbolBudget};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxTree};

use crate::app_error::AppError;

/// Separate aggregate phase quotas; parser peaks are conservatively accumulated.
#[derive(Clone, Copy, Default)]
pub(crate) struct ScriptPackageFrontend {
    pub(crate) source: SourceBudget,
    pub(crate) syntax: LegacySyntaxBudget,
    pub(crate) declaration: DeclarationBudget,
    pub(crate) symbol: SymbolBudget,
}

impl ScriptPackageFrontend {
    pub(crate) fn indexed(&mut self, source: &SourceText<'_>) -> Result<(), AppError> {
        debit([
            (
                &mut self.source.max_source_bytes,
                source.bytes().len() as u64,
            ),
            (&mut self.source.max_lines, source.line_count()),
            (&mut self.source.max_metadata_bytes, source.metadata_bytes()),
        ])
    }

    pub(crate) fn parsed(&mut self, tree: &LegacySyntaxTree<'_>) -> Result<(), AppError> {
        let usage = tree.usage();
        debit([
            (&mut self.syntax.max_nodes, usage.nodes),
            (&mut self.syntax.max_child_edges, usage.child_edges),
            (&mut self.syntax.max_steps, usage.steps),
            (
                &mut self.syntax.max_metadata_bytes,
                usage.peak_metadata_bytes,
            ),
            (
                &mut self.syntax.lexical.max_lexemes,
                usage.lexical.attempted_lexemes,
            ),
            (&mut self.syntax.lexical.max_steps, usage.lexical.steps),
        ])
    }

    pub(crate) fn declared(&mut self, module: &LegacyModuleSyntax<'_, '_>) -> Result<(), AppError> {
        let usage = module.usage();
        debit([
            (&mut self.declaration.max_items, usage.items),
            (&mut self.declaration.max_includes, usage.includes),
            (&mut self.declaration.max_parameters, usage.parameters),
            (&mut self.declaration.max_steps, usage.steps),
            (
                &mut self.declaration.max_metadata_bytes,
                usage.metadata_bytes,
            ),
        ])
    }

    pub(crate) fn bound(
        &mut self,
        symbols: &LegacyModuleSymbols<'_, '_, '_>,
    ) -> Result<(), AppError> {
        let usage = symbols.usage();
        debit([
            (&mut self.symbol.max_functions, usage.functions),
            (&mut self.symbol.max_name_bytes, usage.name_bytes),
            (&mut self.symbol.max_steps, usage.steps),
            (&mut self.symbol.max_metadata_bytes, usage.metadata_bytes),
        ])
    }
}

fn debit<const N: usize>(records: [(&mut u64, u64); N]) -> Result<(), AppError> {
    for (remaining, used) in records {
        *remaining = remaining.checked_sub(used).ok_or(AppError::ScriptFeature(
            "frontend accounting invariant failed",
        ))?;
    }
    Ok(())
}

pub(crate) fn rows<T>(count: usize) -> Result<Vec<T>, AppError> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| AppError::ScriptFeature("package staging allocation failed"))?;
    Ok(rows)
}
