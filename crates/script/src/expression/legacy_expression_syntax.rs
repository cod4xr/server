// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::expression::{
    LegacyExpressionError, LegacyExpressionKind, LegacyInfixOperator, LegacyLogicalOperator,
    LegacyPrefixOperator,
};
use crate::syntax::{LegacySyntaxNode, LegacySyntaxNodeKind};

/// Borrowed typed operator view, retaining the original supplied node and span.
#[derive(Clone, Copy, Debug)]
pub struct LegacyExpressionSyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    kind: LegacyExpressionKind<'tree, 'source>,
}

impl<'tree, 'source> LegacyExpressionSyntax<'tree, 'source> {
    /// Classifies immediate syntax using a constant number of borrowed node reads.
    ///
    /// The fragment expression-entry and single primitive wrappers are recognized.
    /// No allocation, recursion, descendant walk, literal decoding, binding, type
    /// selection, or evaluation occurs. Primary forms remain explicitly opaque.
    /// The original node/span is retained even when a wrapper is removed locally.
    ///
    /// # Errors
    /// Rejects terminals, statements, declarations, and other nonexpression nodes,
    /// or a mismatched pinned production shape. Caller syntax remains unchanged.
    pub fn from_node(
        syntax: LegacySyntaxNode<'tree, 'source>,
    ) -> Result<Self, LegacyExpressionError> {
        let mut node = syntax;
        if production(node) == Some(2) {
            node = child(node, 1, 2)?;
        }
        if production(node) == Some(5) {
            node = child(node, 0, 1)?;
        }
        let kind = classify(node)?;
        Ok(Self { syntax, kind })
    }

    /// Returns the local typed shape without validating any descendants.
    #[must_use]
    pub fn kind(self) -> LegacyExpressionKind<'tree, 'source> {
        self.kind
    }

    /// Returns the original supplied scoped syntax, preserving any wrapper.
    #[must_use]
    pub fn syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the exact original node's half-open source byte span.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.syntax.span()
    }
}

fn classify<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<LegacyExpressionKind<'tree, 'source>, LegacyExpressionError> {
    let rule = production(node).ok_or(LegacyExpressionError::NotExpression)?;
    Ok(match rule {
        6 | 7 => LegacyExpressionKind::Logical {
            operator: if rule == 6 {
                LegacyLogicalOperator::Or
            } else {
                LegacyLogicalOperator::And
            },
            left: child(node, 0, 3)?,
            right: child(node, 2, 3)?,
        },
        8..=23 => LegacyExpressionKind::Infix {
            operator: LegacyInfixOperator::from_production(rule)
                .ok_or(LegacyExpressionError::ProfileMismatch)?,
            left: child(node, 0, 3)?,
            right: child(node, 2, 3)?,
        },
        24 | 25 => LegacyExpressionKind::Prefix {
            operator: if rule == 24 {
                LegacyPrefixOperator::LogicalNot
            } else {
                LegacyPrefixOperator::BitwiseNot
            },
            operand: child(node, 1, 2)?,
        },
        40..=61 => LegacyExpressionKind::Primary { syntax: node },
        _ => return Err(LegacyExpressionError::NotExpression),
    })
}

fn production(node: LegacySyntaxNode<'_, '_>) -> Option<u16> {
    match node.kind() {
        LegacySyntaxNodeKind::Production(rule) => Some(rule.as_u16()),
        _ => None,
    }
}

fn child<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    index: usize,
    count: usize,
) -> Result<LegacySyntaxNode<'tree, 'source>, LegacyExpressionError> {
    let mut children = node.children();
    if children.len() != count {
        return Err(LegacyExpressionError::ProfileMismatch);
    }
    children
        .nth(index)
        .ok_or(LegacyExpressionError::ProfileMismatch)
}
