// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::legacy::LegacyNumber;
use crate::lexical::{LegacyLexeme, LegacyLexemeKind};
use crate::literal::legacy_literal_error;
use crate::literal::number_literal;
use crate::literal::{
    LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource, LegacyString, LegacyStringKind,
};

/// A decoded legacy literal, without a runtime handle, AST node, or bytecode.
pub enum LegacyLiteral {
    /// The legacy undefined value.
    Undefined,
    /// Legacy numeric value; true/false become integer one/zero.
    Number(LegacyNumber),
    /// Owned ordinary/localized byte string, without interning or implicit NUL.
    String(LegacyString),
}

impl LegacyLiteral {
    /// Decodes a supported literal lexeme under explicit per-literal budgets.
    ///
    /// String escapes match the pinned actions (r/t/n, otherwise escaped byte).
    /// Decode storage is sized/admitted before fallible allocation. No fixed
    /// 8191-byte stack buffer or 16-bit string-table handle is reproduced.
    /// Integer/finite binary32 decimal literals use the defined legacy value
    /// domain; unary signs remain separate syntax. Numeric parsing is locale-free.
    ///
    /// # Errors
    /// Rejects other token kinds, bad spellings, budget/allocation failures, integer
    /// overflow, and non-finite float results. No partial value is returned or
    /// source mutated. Undefined C conversion/locale-dependent behavior is not
    /// claimed compatible. Modern numeric types/escapes remain separate work.
    pub fn decode(
        lexeme: LegacyLexeme<'_>,
        budget: LegacyLiteralBudget,
    ) -> Result<Self, LegacyLiteralError> {
        let LegacyLexemeKind::Token(code) = lexeme.kind() else {
            return Err(LegacyLiteralError::WrongToken);
        };
        let bytes = lexeme.as_bytes();
        legacy_literal_error::check(
            LegacyLiteralResource::EncodedBytes,
            bytes.len() as u64,
            budget.max_encoded_bytes,
        )?;
        match code.as_u16() {
            0x103 => LegacyString::decode(bytes, LegacyStringKind::Plain, budget.max_string_bytes)
                .map(Self::String),
            0x104 => {
                LegacyString::decode(bytes, LegacyStringKind::Localized, budget.max_string_bytes)
                    .map(Self::String)
            }
            0x11f => number_literal::integer(bytes).map(Self::Number),
            0x120 => number_literal::float(bytes).map(Self::Number),
            0x12a => Ok(Self::Undefined),
            0x14f => Ok(Self::Number(LegacyNumber::Integer(0))),
            0x150 => Ok(Self::Number(LegacyNumber::Integer(1))),
            _ => Err(LegacyLiteralError::WrongToken),
        }
    }
}

impl fmt::Debug for LegacyLiteral {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undefined => formatter.write_str("LegacyLiteral::Undefined"),
            Self::Number(LegacyNumber::Integer(_)) => {
                formatter.write_str("LegacyLiteral::Integer(..)")
            }
            Self::Number(LegacyNumber::Float(_)) => formatter.write_str("LegacyLiteral::Float(..)"),
            Self::String(string) => string.fmt(formatter),
        }
    }
}
