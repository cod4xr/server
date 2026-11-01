// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::expression::ExpressionListErrorKind;
use crate::expression::legacy_syntax_profile;
use crate::syntax::LegacySyntaxNode;

pub(crate) struct ExpressionListCursor<'tree, 'source> {
    current: Option<LegacySyntaxNode<'tree, 'source>>,
}

impl<'tree, 'source> ExpressionListCursor<'tree, 'source> {
    pub(crate) fn new(node: LegacySyntaxNode<'tree, 'source>) -> Self {
        Self {
            current: Some(node),
        }
    }

    pub(crate) fn pending(&self) -> bool {
        self.current.is_some()
    }

    pub(crate) fn advance(
        &mut self,
    ) -> Result<Option<LegacySyntaxNode<'tree, 'source>>, ExpressionListErrorKind> {
        let node = self
            .current
            .ok_or(ExpressionListErrorKind::InvalidStructure)?;
        let child = |index, count| {
            legacy_syntax_profile::child(node, index, count)
                .map_err(|_| ExpressionListErrorKind::InvalidStructure)
        };
        match legacy_syntax_profile::production(node) {
            Some(113) => {
                self.current = Some(child(0, 1)?);
                Ok(None)
            }
            Some(114) => {
                if node.children().next().is_some() {
                    return Err(ExpressionListErrorKind::InvalidStructure);
                }
                self.current = None;
                Ok(None)
            }
            Some(111) => {
                let expression = child(2, 3)?;
                self.current = Some(child(0, 3)?);
                Ok(Some(expression))
            }
            Some(112) => {
                let expression = child(0, 1)?;
                self.current = None;
                Ok(Some(expression))
            }
            _ => Err(ExpressionListErrorKind::ExpectedList),
        }
    }
}
