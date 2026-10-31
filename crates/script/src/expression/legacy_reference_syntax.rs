// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::expression::legacy_syntax_profile;
use crate::expression::{
    LegacyExpressionError, LegacyPrimaryKind, LegacyPrimarySyntax, LegacyReferenceKind,
};
use crate::syntax::LegacySyntaxNode;

/// Constant-cost borrowed reference syntax, without selecting runtime storage.
#[derive(Clone, Copy, Debug)]
pub struct LegacyReferenceSyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    kind: LegacyReferenceKind<'tree, 'source>,
}

impl<'tree, 'source> LegacyReferenceSyntax<'tree, 'source> {
    /// Classifies a direct reference or a reference primary/wrapper locally.
    ///
    /// Performs bounded checked child reads without allocating, traversing
    /// descendants, folding names, resolving scopes/objects, or evaluating data.
    /// Dollar and reserved self-member forms remain distinct, with no implied
    /// runtime availability, authority, type, or writable-slot guarantee.
    ///
    /// # Errors
    /// Rejects nonreference nodes and mismatched pinned child profiles. Source
    /// state remains unchanged; no partial view or runtime mutation is exposed.
    pub fn from_node(
        syntax: LegacySyntaxNode<'tree, 'source>,
    ) -> Result<Self, LegacyExpressionError> {
        let node = if matches!(legacy_syntax_profile::production(syntax), Some(62..=66)) {
            syntax
        } else {
            let primary = match LegacyPrimarySyntax::from_node(syntax) {
                Ok(primary) => primary,
                Err(LegacyExpressionError::NotExpression | LegacyExpressionError::NotPrimary) => {
                    return Err(LegacyExpressionError::NotReference);
                }
                Err(error) => return Err(error),
            };
            let LegacyPrimaryKind::Reference { syntax } = primary.kind() else {
                return Err(LegacyExpressionError::NotReference);
            };
            syntax
        };
        let child = |index, count| legacy_syntax_profile::child(node, index, count);
        let token = |index, count| {
            child(index, count)?
                .lexeme()
                .ok_or(LegacyExpressionError::ProfileMismatch)
        };
        let kind = match legacy_syntax_profile::production(node) {
            Some(62) => LegacyReferenceKind::Field {
                object: child(0, 3)?,
                name: token(2, 3)?,
            },
            Some(63) => LegacyReferenceKind::Index {
                object: child(0, 4)?,
                index: child(2, 4)?,
            },
            Some(64) => LegacyReferenceKind::Name { name: token(0, 1)? },
            Some(65) => LegacyReferenceKind::DollarName { name: token(1, 2)? },
            Some(66) => LegacyReferenceKind::SelfMember {
                object: child(0, 3)?,
            },
            _ => return Err(LegacyExpressionError::NotReference),
        };
        Ok(Self { syntax, kind })
    }

    /// Returns the local role; descendants, names, types, and mutability stay unresolved.
    #[must_use]
    pub fn kind(self) -> LegacyReferenceKind<'tree, 'source> {
        self.kind
    }

    /// Returns the supplied scoped syntax, preserving any original primary wrapper.
    #[must_use]
    pub fn syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the supplied node's original half-open source byte span.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.syntax.span()
    }
}
