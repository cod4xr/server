// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::lexical::{LegacyLexeme, LegacyTokenCode};
use crate::syntax::LegacySyntaxMode;

#[derive(Clone, Copy)]
pub(super) enum Lookahead<'source> {
    Selector(LegacySyntaxMode),
    Source {
        code: LegacyTokenCode,
        lexeme: LegacyLexeme<'source>,
    },
    Eof(u64),
}

impl Lookahead<'_> {
    pub(super) fn code(self) -> u16 {
        match self {
            Self::Selector(mode) => mode.selector(),
            Self::Source { code, .. } => code.as_u16(),
            Self::Eof(_) => 0,
        }
    }

    pub(super) fn offset(self) -> u64 {
        match self {
            Self::Selector(_) => 0,
            Self::Source { lexeme, .. } => lexeme.span().start,
            Self::Eof(offset) => offset,
        }
    }
}
