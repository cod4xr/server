// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::lexical::LegacyLexUsage;
use crate::source::SourceText;
use crate::syntax::legacy_parser;
use crate::syntax::syntax_arena::SyntaxArena;
use crate::syntax::{LegacySyntaxBudget, LegacySyntaxError, LegacySyntaxMode, LegacySyntaxNode};

/// Logical successful parse accounting; excludes allocator overhead/spare capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacySyntaxUsage {
    /// Real shifted source tokens; excludes the selector and EOF.
    pub source_tokens: u64,
    /// Applied grammar reductions.
    pub reductions: u64,
    /// Flat nodes, including terminals and empty productions.
    pub nodes: u64,
    /// Ordered child edges.
    pub child_edges: u64,
    /// Attempted grammar actions, including the accepting action.
    pub steps: u64,
    /// Maximum live parser stack entries, including the state-zero sentinel.
    pub peak_stack_entries: u64,
    /// Peak live logical node/edge/stack metadata bytes during construction.
    pub peak_metadata_bytes: u64,
    /// Scanner work, including trivia and lookahead.
    pub lexical: LegacyLexUsage,
}

/// Flat structural legacy tree borrowing original bytes, with iterative destruction.
///
/// Trivia remains in the original source bytes, not separate child nodes. This is
/// a concrete grammar tree, not a semantic AST, compiler, literal validator, or VM.
/// Children refer only to older nodes; no native pointers or externally forged IDs
/// enter the arena. Dropping this value does not recurse through script nesting.
pub struct LegacySyntaxTree<'source> {
    pub(super) bytes: &'source [u8],
    pub(super) mode: LegacySyntaxMode,
    pub(super) arena: SyntaxArena<'source>,
    pub(super) root: usize,
    pub(super) usage: LegacySyntaxUsage,
}

impl<'source> LegacySyntaxTree<'source> {
    /// Parses only structural syntax under the pinned legacy grammar.
    ///
    /// The selector is injected as a control terminal with span 0..0; it does not
    /// alter source bytes/offsets. Source indexing is admitted separately. Limits
    /// bound scan work, grammar work, stack, and tree growth before allocation.
    ///
    /// # Errors
    /// Fails at the first needed lexical/syntax error or resource failure, without
    /// returning a partial tree or modifying the input. Original error recovery
    /// and semantic actions are not executed; success does not imply runnable GSC.
    pub fn parse_structure(
        source: &SourceText<'source>,
        mode: LegacySyntaxMode,
        budget: LegacySyntaxBudget,
    ) -> Result<Self, LegacySyntaxError> {
        legacy_parser::parse(source, mode, budget)
    }

    /// Returns the entry production's scoped view.
    #[must_use]
    pub fn root(&self) -> LegacySyntaxNode<'_, 'source> {
        LegacySyntaxNode {
            tree: self,
            index: self.root,
        }
    }

    /// Iterates nodes in construction order: each child precedes its parent.
    #[must_use]
    pub fn nodes(
        &self,
    ) -> impl ExactSizeIterator<Item = LegacySyntaxNode<'_, 'source>> + DoubleEndedIterator {
        (0..self.arena.nodes.len()).map(|index| LegacySyntaxNode { tree: self, index })
    }

    /// Borrows all unchanged original source bytes, including comments/trivia.
    #[must_use]
    pub fn source_bytes(&self) -> &'source [u8] {
        self.bytes
    }

    /// Returns the selected legacy entry grammar, not a modern language version.
    #[must_use]
    pub fn mode(&self) -> LegacySyntaxMode {
        self.mode
    }

    /// Returns successful logical peak/work accounting, without source contents.
    #[must_use]
    pub fn usage(&self) -> LegacySyntaxUsage {
        self.usage
    }
}

impl fmt::Debug for LegacySyntaxTree<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacySyntaxTree")
            .field("mode", &self.mode)
            .field("bytes", &self.bytes.len())
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
