// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::expression::LegacyFunctionNameSyntax;
use crate::syntax::LegacySyntaxNode;

/// Invocation spelling; classifying it creates no execution or scheduling action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyInvocationKind {
    /// Ordinary immediate invocation syntax.
    Immediate,
    /// Legacy thread invocation syntax; no OS/script thread is created here.
    Thread,
}

/// Borrowed callee role, before name binding or indirect expression evaluation.
#[derive(Clone, Copy, Debug)]
pub enum LegacyCalleeKind<'tree, 'source> {
    /// Original local/external module and function-name tokens.
    Named(LegacyFunctionNameSyntax<'tree, 'source>),
    /// The unevaluated expression inside a double-bracket target.
    Indirect {
        /// Original expression syntax; no target or runtime value is resolved.
        expression: LegacySyntaxNode<'tree, 'source>,
    },
}
