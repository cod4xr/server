// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::lexical::LegacyLexeme;
use crate::syntax::LegacySyntaxNode;

/// Typed function declaration, borrowing its tree and owning only parameter views.
///
/// Names remain exact source spelling. Case folding/interning, duplicate checks,
/// body lowering, local scopes, and executable semantics belong to later phases.
pub struct LegacyFunctionSyntax<'tree, 'source> {
    pub(super) name: LegacyLexeme<'source>,
    pub(super) parameters: Vec<LegacyLexeme<'source>>,
    pub(super) body: LegacySyntaxNode<'tree, 'source>,
    pub(super) span: Range<u64>,
}

impl<'tree, 'source> LegacyFunctionSyntax<'tree, 'source> {
    /// Borrows the original function identifier, without normalization/interning.
    #[must_use]
    pub fn name(&self) -> LegacyLexeme<'source> {
        self.name
    }

    /// Borrows ordered parameter identifiers, preserving duplicates and raw case.
    #[must_use]
    pub fn parameters(&self) -> &[LegacyLexeme<'source>] {
        &self.parameters
    }

    /// Returns the raw statement-list node, excluding the surrounding braces.
    /// Empty-production anchors may include leading trivia in its byte span.
    #[must_use]
    pub fn body(&self) -> LegacySyntaxNode<'tree, 'source> {
        self.body
    }

    /// Returns the complete function declaration's original byte span.
    #[must_use]
    pub fn span(&self) -> Range<u64> {
        self.span.clone()
    }
}

impl fmt::Debug for LegacyFunctionSyntax<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyFunctionSyntax")
            .field("span", &self.span)
            .field("parameters", &self.parameters.len())
            .finish_non_exhaustive()
    }
}

/// Ordered top-level syntax items, without changing developer-code policy.
#[derive(Debug)]
pub enum LegacyModuleItem<'tree, 'source> {
    /// Function signature with its original structural body.
    Function(LegacyFunctionSyntax<'tree, 'source>),
    /// Original animation-tree string lexeme, not a resolved engine asset.
    UsingAnimTree(LegacyLexeme<'source>),
    /// Opening developer-code marker; balancing/filtering belongs to compilation.
    DeveloperBegin {
        /// Original marker's source byte offset.
        offset: u64,
    },
    /// Closing developer-code marker; does not imply a matching opening marker.
    DeveloperEnd {
        /// Original marker's source byte offset.
        offset: u64,
    },
}
