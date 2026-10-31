// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::statement::StatementListErrorKind;
use crate::statement::statement_profile;
use crate::syntax::LegacySyntaxNode;

pub(super) struct StatementListCursor<'tree, 'source> {
    next: Option<LegacySyntaxNode<'tree, 'source>>,
}

impl<'tree, 'source> StatementListCursor<'tree, 'source> {
    pub(super) fn new(
        syntax: LegacySyntaxNode<'tree, 'source>,
    ) -> Result<Self, StatementListErrorKind> {
        if !matches!(statement_profile::production(syntax), Some(109 | 110)) {
            return Err(StatementListErrorKind::NotStatementList);
        }
        Ok(Self { next: Some(syntax) })
    }

    pub(super) fn pending(&self) -> bool {
        self.next.is_some()
    }

    pub(super) fn advance(
        &mut self,
    ) -> Result<Option<LegacySyntaxNode<'tree, 'source>>, StatementListErrorKind> {
        let node = self
            .next
            .take()
            .ok_or(StatementListErrorKind::ProfileMismatch)?;
        match statement_profile::production(node) {
            Some(109) => {
                let previous = statement_profile::child(node, 0, 2)
                    .map_err(|_| StatementListErrorKind::ProfileMismatch)?;
                let statement = statement_profile::child(node, 1, 2)
                    .map_err(|_| StatementListErrorKind::ProfileMismatch)?;
                self.next = Some(previous);
                Ok(Some(statement))
            }
            Some(110) if node.children().next().is_none() => Ok(None),
            _ => Err(StatementListErrorKind::ProfileMismatch),
        }
    }
}
