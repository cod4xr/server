// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Rejected per-literal resource without source contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLiteralResource {
    /// Complete encoded lexeme bytes.
    EncodedBytes,
    /// Decoded string payload bytes.
    StringBytes,
}

/// Sanitized literal failure; callers retain the original source span separately.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLiteralError {
    /// The lexeme is trivia or not a supported literal terminal.
    WrongToken,
    /// Source spelling does not match the selected literal kind.
    InvalidSpelling,
    /// Integer overflow or a non-finite decimal float lies outside this decoder's domain.
    NumberOutOfRange,
    /// A configured encoded/decoded byte budget was exceeded.
    BudgetExceeded(LegacyLiteralResource),
    /// Fallible decoded string allocation failed.
    AllocationFailed,
}

impl fmt::Display for LegacyLiteralError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("legacy script literal error: ")?;
        match self {
            Self::WrongToken => formatter.write_str("expected a supported literal"),
            Self::InvalidSpelling => formatter.write_str("invalid literal spelling"),
            Self::NumberOutOfRange => formatter.write_str("numeric literal is out of range"),
            Self::BudgetExceeded(resource) => write!(formatter, "budget exceeded: {resource:?}"),
            Self::AllocationFailed => formatter.write_str("string allocation failed"),
        }
    }
}

impl Error for LegacyLiteralError {}

pub(super) fn check(
    resource: LegacyLiteralResource,
    size: u64,
    maximum: u64,
) -> Result<(), LegacyLiteralError> {
    if size > maximum {
        return Err(LegacyLiteralError::BudgetExceeded(resource));
    }
    Ok(())
}
