// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::expression::legacy_syntax_profile;
use crate::expression::{
    ExpressionListBudget, ExpressionListError, LegacyCalleeKind, LegacyExpressionError,
    LegacyExpressionListSyntax, LegacyFunctionNameSyntax, LegacyInvocationKind, LegacyPrimaryKind,
    LegacyPrimarySyntax,
};
use crate::syntax::LegacySyntaxNode;

/// Borrowed local invocation roles, without argument/receiver evaluation or binding.
#[derive(Clone, Copy, Debug)]
pub struct LegacyCallSyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    receiver: Option<LegacySyntaxNode<'tree, 'source>>,
    kind: LegacyInvocationKind,
    callee: LegacyCalleeKind<'tree, 'source>,
    arguments: LegacySyntaxNode<'tree, 'source>,
}

impl<'tree, 'source> LegacyCallSyntax<'tree, 'source> {
    /// Classifies a direct invocation production or call primary/wrapper locally.
    ///
    /// Construction is allocation-free and constant-cost. Arguments, receiver,
    /// indirect expressions, names, and module paths remain unresolved. No
    /// function, builtin, host service, or thread runs and no runtime IDs are made.
    ///
    /// # Errors
    /// Rejects noncall nodes and mismatched pinned invocation/callee shapes without
    /// changing source/tree state or exposing a partial view.
    pub fn from_node(
        syntax: LegacySyntaxNode<'tree, 'source>,
    ) -> Result<Self, LegacyExpressionError> {
        let node = if matches!(legacy_syntax_profile::production(syntax), Some(38 | 39)) {
            syntax
        } else {
            let primary = match LegacyPrimarySyntax::from_node(syntax) {
                Ok(primary) => primary,
                Err(LegacyExpressionError::NotExpression | LegacyExpressionError::NotPrimary) => {
                    return Err(LegacyExpressionError::NotCall);
                }
                Err(error) => return Err(error),
            };
            let LegacyPrimaryKind::Call { syntax } = primary.kind() else {
                return Err(LegacyExpressionError::NotCall);
            };
            syntax
        };
        let child = |index, count| legacy_syntax_profile::child(node, index, count);
        let (receiver, target, arguments) = match legacy_syntax_profile::production(node) {
            Some(38) => (None, child(0, 4)?, child(2, 4)?),
            Some(39) => (Some(child(0, 5)?), child(1, 5)?, child(3, 5)?),
            _ => return Err(LegacyExpressionError::NotCall),
        };
        let (kind, target) = match legacy_syntax_profile::production(target) {
            Some(36) => (
                LegacyInvocationKind::Immediate,
                legacy_syntax_profile::child(target, 0, 1)?,
            ),
            Some(37) => (
                LegacyInvocationKind::Thread,
                legacy_syntax_profile::child(target, 1, 2)?,
            ),
            _ => return Err(LegacyExpressionError::ProfileMismatch),
        };
        let callee = match legacy_syntax_profile::production(target) {
            Some(34) => LegacyCalleeKind::Named(LegacyFunctionNameSyntax::from_node(
                legacy_syntax_profile::child(target, 0, 1)?,
            )?),
            Some(35) => LegacyCalleeKind::Indirect {
                expression: legacy_syntax_profile::child(target, 2, 5)?,
            },
            _ => return Err(LegacyExpressionError::ProfileMismatch),
        };
        Ok(Self {
            syntax,
            receiver,
            kind,
            callee,
            arguments,
        })
    }

    /// Returns an unresolved method receiver, or None for an ordinary function call.
    #[must_use]
    pub fn receiver(self) -> Option<LegacySyntaxNode<'tree, 'source>> {
        self.receiver
    }

    /// Returns immediate/thread spelling without creating an execution action.
    #[must_use]
    pub fn kind(self) -> LegacyInvocationKind {
        self.kind
    }

    /// Returns the named/indirect target role without resolving it.
    #[must_use]
    pub fn callee(self) -> LegacyCalleeKind<'tree, 'source> {
        self.callee
    }

    /// Returns the original optional expression-list node without traversing it.
    #[must_use]
    pub fn arguments_syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.arguments
    }

    /// Builds a separately admitted source-order argument index; evaluates nothing.
    ///
    /// # Errors
    /// Propagates list shape/resource/allocation errors. Source order does not
    /// establish native argument/receiver evaluation or runtime parameter capacity.
    pub fn lower_arguments(
        self,
        budget: ExpressionListBudget,
    ) -> Result<LegacyExpressionListSyntax<'tree, 'source>, ExpressionListError> {
        LegacyExpressionListSyntax::lower(self.arguments, budget)
    }

    /// Returns the original supplied node, preserving any primary wrapper.
    #[must_use]
    pub fn syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the original supplied node's half-open source byte span.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.syntax.span()
    }
}
