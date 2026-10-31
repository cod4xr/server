// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::lexical::{LegacyLexeme, LegacyTokenCode};
use crate::syntax::{LegacySyntaxMode, LegacySyntaxTree};

/// Pinned grammar production number, separate from runtime function/object IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyProductionCode(pub(super) u16);

impl LegacyProductionCode {
    /// Returns the source grammar's production number for conformance/tooling.
    #[must_use]
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Structural category; no semantic action, literal conversion, or binding implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacySyntaxNodeKind {
    /// Synthetic parser entry selector with an empty source span.
    Selector(LegacySyntaxMode),
    /// Explicit EOF with an empty span at the original source length.
    EndOfInput,
    /// Original source terminal, excluding trivia.
    Terminal(LegacyTokenCode),
    /// Ordered reduction under the pinned legacy grammar.
    Production(LegacyProductionCode),
}

#[derive(Clone, Copy)]
pub(super) enum NodeData<'source> {
    Selector(LegacySyntaxMode),
    EndOfInput,
    Terminal {
        code: LegacyTokenCode,
        lexeme: LegacyLexeme<'source>,
    },
    Production(u16),
}

pub(super) struct Node<'source> {
    pub(super) data: NodeData<'source>,
    pub(super) span: Range<u64>,
    pub(super) children: Range<usize>,
}

/// Scoped borrowed node view; callers cannot forge IDs or mix trees' indices.
#[derive(Clone, Copy)]
pub struct LegacySyntaxNode<'tree, 'source> {
    pub(super) tree: &'tree LegacySyntaxTree<'source>,
    pub(super) index: usize,
}

impl<'tree, 'source> LegacySyntaxNode<'tree, 'source> {
    /// Returns structural identity without interpreting script semantics.
    #[must_use]
    pub fn kind(self) -> LegacySyntaxNodeKind {
        match self.tree.arena.nodes[self.index].data {
            NodeData::Selector(mode) => LegacySyntaxNodeKind::Selector(mode),
            NodeData::EndOfInput => LegacySyntaxNodeKind::EndOfInput,
            NodeData::Production(rule) => {
                LegacySyntaxNodeKind::Production(LegacyProductionCode(rule))
            }
            NodeData::Terminal { code, .. } => LegacySyntaxNodeKind::Terminal(code),
        }
    }

    /// Returns the half-open byte span in the original source, without a prefix.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.tree.arena.nodes[self.index].span.clone()
    }

    /// Borrows exact original terminals for later literal decoding/name binding.
    #[must_use]
    pub fn lexeme(self) -> Option<LegacyLexeme<'source>> {
        match self.tree.arena.nodes[self.index].data {
            NodeData::Terminal { lexeme, .. } => Some(lexeme),
            _ => None,
        }
    }

    /// Iterates immediate children in source grammar order, without recursion.
    #[must_use]
    pub fn children(self) -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + 'tree {
        self.tree.arena.edges[self.tree.arena.nodes[self.index].children.clone()]
            .iter()
            .map(move |index| Self {
                tree: self.tree,
                index: *index,
            })
    }
}

impl fmt::Debug for LegacySyntaxNode<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacySyntaxNode")
            .field("kind", &self.kind())
            .field("span", &self.span())
            .field("children", &self.children().len())
            .finish_non_exhaustive()
    }
}
