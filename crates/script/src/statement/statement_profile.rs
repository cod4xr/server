// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::statement::LegacyStatementError;
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
) -> Result<LegacySyntaxNode<'tree, 'source>, LegacyStatementError> {
    let mut children = node.children();
    if children.len() != count {
        return Err(LegacyStatementError::ProfileMismatch);
    }
    children
        .nth(index)
        .ok_or(LegacyStatementError::ProfileMismatch)
}

pub(super) fn strip_wrappers<'tree, 'source>(
    mut node: LegacySyntaxNode<'tree, 'source>,
) -> Result<LegacySyntaxNode<'tree, 'source>, LegacyStatementError> {
    // At most 108 -> 97 -> 94 -> statement; 96 is the optional for-update wrapper.
    for _ in 0..4 {
        let count = match production(node) {
            Some(94 | 96 | 108) => 1,
            Some(97) => 2,
            _ => return Ok(node),
        };
        node = child(node, 0, count)?;
    }
    Err(LegacyStatementError::ProfileMismatch)
}
