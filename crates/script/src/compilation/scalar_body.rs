// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_compile_error;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::declaration::LegacyFunctionSyntax;
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
    let mut statement = child(body, 1, 2)?;
    for (rule, count) in [(108, 1), (97, 2), (94, 1)] {
        if production(statement) != Some(rule) {
            return Err(scalar_compile_error::at(
                statement.span(),
                ScalarCompileErrorKind::UnsupportedBody,
            ));
        }
        statement = child(statement, 0, count)?;
    }
    match production(statement) {
        Some(68) => Ok(ScalarBody {
            expression: Some(child(statement, 1, 2)?),
            return_span: statement.span(),
        }),
        Some(69) => {
            child(statement, 0, 1)?;
            Ok(ScalarBody {
                expression: None,
                return_span: statement.span(),
            })
        }
        _ => Err(scalar_compile_error::at(
            statement.span(),
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
