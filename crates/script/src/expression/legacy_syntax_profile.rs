// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::expression::LegacyExpressionError;
use crate::syntax::{LegacySyntaxNode, LegacySyntaxNodeKind};

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
) -> Result<LegacySyntaxNode<'tree, 'source>, LegacyExpressionError> {
    let mut children = node.children();
    if children.len() != count {
        return Err(LegacyExpressionError::ProfileMismatch);
    }
    children
        .nth(index)
        .ok_or(LegacyExpressionError::ProfileMismatch)
}
