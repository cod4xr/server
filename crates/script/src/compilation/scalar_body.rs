// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_compile_error;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::declaration::LegacyFunctionSyntax;
use crate::statement::{LegacyStatementError, LegacyStatementKind, LegacyStatementSyntax};
use crate::syntax::{LegacySyntaxNode, LegacySyntaxNodeKind};

pub(super) struct ScalarBody<'tree, 'source> {
    pub(super) expression: Option<LegacySyntaxNode<'tree, 'source>>,
    pub(super) return_span: Range<u64>,
}

pub(super) fn lower<'tree, 'source>(
    function: &LegacyFunctionSyntax<'tree, 'source>,
) -> Result<ScalarBody<'tree, 'source>, ScalarCompileError> {
    if !function.parameters().is_empty() {
        return Err(scalar_compile_error::at(
            function.span(),
            ScalarCompileErrorKind::UnsupportedParameters,
        ));
    }
    let body = function.body();
    if production(body) == Some(110) {
        if body.children().next().is_some() {
            return Err(scalar_compile_error::at(
                body.span(),
                ScalarCompileErrorKind::InvalidStructure,
            ));
        }
        return Ok(ScalarBody {
            expression: None,
            return_span: body.span(),
        });
    }
    if production(body) != Some(109) {
        return Err(scalar_compile_error::at(
            body.span(),
            ScalarCompileErrorKind::UnsupportedBody,
        ));
    }
    let previous = child(body, 0, 2)?;
    if production(previous) != Some(110) || previous.children().next().is_some() {
        return Err(scalar_compile_error::at(
            body.span(),
            ScalarCompileErrorKind::UnsupportedBody,
        ));
    }
    let node = child(body, 1, 2)?;
    let statement = LegacyStatementSyntax::from_node(node).map_err(|error| {
        let kind = match error {
            LegacyStatementError::NotStatement => ScalarCompileErrorKind::UnsupportedBody,
            LegacyStatementError::ProfileMismatch => ScalarCompileErrorKind::InvalidStructure,
        };
        scalar_compile_error::at(node.span(), kind)
    })?;
    match statement.kind() {
        LegacyStatementKind::Return { value } => Ok(ScalarBody {
            expression: value,
            return_span: statement.statement_syntax().span(),
        }),
        _ => Err(scalar_compile_error::at(
            statement.statement_syntax().span(),
            ScalarCompileErrorKind::UnsupportedBody,
        )),
    }
}

pub(super) fn production(node: LegacySyntaxNode<'_, '_>) -> Option<u16> {
    match node.kind() {
        LegacySyntaxNodeKind::Production(rule) => Some(rule.as_u16()),
        _ => None,
    }
}

pub(super) fn child<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    index: usize,
    count: usize,
) -> Result<LegacySyntaxNode<'tree, 'source>, ScalarCompileError> {
    let mut children = node.children();
    if children.len() != count {
        return Err(scalar_compile_error::at(
            node.span(),
            ScalarCompileErrorKind::InvalidStructure,
        ));
    }
    children.nth(index).ok_or_else(|| {
        scalar_compile_error::at(node.span(), ScalarCompileErrorKind::InvalidStructure)
    })
}
