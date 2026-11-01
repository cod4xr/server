// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_body;
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::statement::{LegacyStatementKind, LegacyStatementSyntax};
use crate::syntax::LegacySyntaxNode;

pub(super) fn validate(node: LegacySyntaxNode<'_, '_>) -> Result<(), FunctionCompileError> {
    let view = LegacyStatementSyntax::from_node(node).map_err(|_| invalid(node))?;
    if matches!(
        view.kind(),
        LegacyStatementKind::Empty
            | LegacyStatementKind::Assignment { .. }
            | LegacyStatementKind::CompoundAssignment { .. }
            | LegacyStatementKind::Update { .. }
    ) {
        Ok(())
    } else {
        Err(function_compile_error::at(
            node.span(),
            FunctionCompileErrorKind::UnsupportedForClause,
        ))
    }
}

pub(super) fn condition<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<Option<LegacySyntaxNode<'tree, 'source>>, FunctionCompileError> {
    match function_body::production(node) {
        Some(26) => function_body::child(node, 0, 1).map(Some),
        Some(27) if node.children().len() == 0 => Ok(None),
        _ => Err(invalid(node)),
    }
}

pub(super) fn update<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<Option<LegacySyntaxNode<'tree, 'source>>, FunctionCompileError> {
    match function_body::production(node) {
        Some(95) if node.children().len() == 0 => Ok(None),
        Some(96) => {
            let statement = function_body::child(node, 0, 1)?;
            validate(statement)?;
            Ok(Some(statement))
        }
        _ => Err(invalid(node)),
    }
}

fn invalid(node: LegacySyntaxNode<'_, '_>) -> FunctionCompileError {
    function_compile_error::at(node.span(), FunctionCompileErrorKind::InvalidStructure)
}
