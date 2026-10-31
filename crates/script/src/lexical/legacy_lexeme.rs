// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::lexical::LegacyLexErrorKind;
use crate::lexical::legacy_tables;

/// Original grammar terminal code; this is a token kind, not a string/object ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyTokenCode(u16);

impl LegacyTokenCode {
    /// Returns the pinned grammar's terminal code without interpreting its value.
    #[must_use]
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Raw lexeme classification; trivia is retained for tooling and exact spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLexemeKind {
    /// Spaces/newlines under the source's rules, without normalization.
    Whitespace,
    /// A single-line comment; any following newline is its own lexeme.
    LineComment,
    /// An opening block-comment delimiter.
    BlockCommentStart,
    /// Block-comment contents; newlines remain whitespace lexemes.
    BlockCommentBody,
    /// A closing block-comment delimiter.
    BlockCommentEnd,
    /// A terminal code; literal/identifier contents are not decoded or interned.
    Token(LegacyTokenCode),
}

/// Borrowed exact raw source lexeme, including literal/comment delimiters.
#[derive(Clone, Copy)]
pub struct LegacyLexeme<'source> {
    kind: LegacyLexemeKind,
    rule: u8,
    offset: u64,
    bytes: &'source [u8],
}

impl<'source> LegacyLexeme<'source> {
    /// Returns the raw source category/terminal without parsing semantics.
    #[must_use]
    pub fn kind(self) -> LegacyLexemeKind {
        self.kind
    }

    /// Returns the pinned lexical action number for source-level conformance work.
    #[must_use]
    pub fn source_rule(self) -> u8 {
        self.rule
    }

    /// Returns exact borrowed bytes; these may contain private script data.
    #[must_use]
    pub fn as_bytes(self) -> &'source [u8] {
        self.bytes
    }

    /// Returns the half-open source byte span; no implicit prefix/suffix is added.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.offset..self.offset + self.bytes.len() as u64
    }

    pub(super) fn new(
        rule: u8,
        offset: u64,
        bytes: &'source [u8],
    ) -> Result<Self, LegacyLexErrorKind> {
        let kind = match rule {
            1 | 4 => LegacyLexemeKind::Whitespace,
            2 => LegacyLexemeKind::BlockCommentEnd,
            3 => LegacyLexemeKind::BlockCommentBody,
            5 => LegacyLexemeKind::LineComment,
            6 => LegacyLexemeKind::BlockCommentStart,
            7..=92 => {
                let code = *legacy_tables::TOKEN_CODES
                    .get(usize::from(rule))
                    .ok_or(LegacyLexErrorKind::InvalidTable)?;
                if code == 0 {
                    return Err(LegacyLexErrorKind::InvalidTable);
                }
                LegacyLexemeKind::Token(LegacyTokenCode(code))
            }
            _ => return Err(LegacyLexErrorKind::InvalidByte),
        };
        Ok(Self {
            kind,
            rule,
            offset,
            bytes,
        })
    }
}

impl fmt::Debug for LegacyLexeme<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyLexeme")
            .field("kind", &self.kind)
            .field("offset", &self.offset)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
