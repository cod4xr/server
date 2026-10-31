// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::lexical::LegacyLexeme;

/// Unresolved animation-tree directive with original literal and complete span.
#[derive(Clone, Copy)]
pub struct LegacyAnimTreeSyntax<'source> {
    pub(super) name: LegacyLexeme<'source>,
    pub(super) start: u64,
    pub(super) end: u64,
}

impl<'source> LegacyAnimTreeSyntax<'source> {
    /// Borrows the original string lexeme without decoding or resolving an asset.
    #[must_use]
    pub fn name(self) -> LegacyLexeme<'source> {
        self.name
    }

    /// Returns the complete directive span, including keyword and semicolon.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.start..self.end
    }
}

impl fmt::Debug for LegacyAnimTreeSyntax<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyAnimTreeSyntax")
            .field("span", &self.span())
            .finish_non_exhaustive()
    }
}
