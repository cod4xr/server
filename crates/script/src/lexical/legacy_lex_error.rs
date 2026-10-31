// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Exhausted logical scanner resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLexResource {
    /// Returned lexemes, including trivia.
    Lexemes,
    /// DFA transition/fallback attempts.
    Steps,
    /// Bytes in one borrowed lexeme.
    LexemeBytes,
}

/// Sanitized lexical failure without script contents or paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLexErrorKind {
    /// An embedded NUL is rejected rather than silently ending source input.
    NulByte,
    /// The source DFA identifies a bad/unrecognized token or incomplete token.
    InvalidByte,
    /// EOF was reached before closing a block comment.
    UnterminatedComment,
    /// A configured scan budget was exhausted.
    BudgetExceeded(LegacyLexResource),
    /// A generated-table invariant failed; this is not a valid input outcome.
    InvalidTable,
}

/// One lexical failure at a zero-based source byte offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyLexError {
    /// Fixed failure category; no received source text is stored.
    pub kind: LegacyLexErrorKind,
    /// Offending byte or opening-comment offset relative to this source.
    pub offset: u64,
}

impl fmt::Display for LegacyLexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "legacy script lexical error at byte {}: {:?}",
            self.offset, self.kind
        )
    }
}

impl Error for LegacyLexError {}
