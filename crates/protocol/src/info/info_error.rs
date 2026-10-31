// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// A rejected info-string operation; no keys or values are retained in errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InfoError {
    /// Encoded length exceeds its protocol profile or caller capacity.
    TooLong,
    /// An empty key or dangling delimiter has no valid key/value interpretation.
    MissingKey,
    /// A key is not followed by its value delimiter.
    MissingValue,
    /// NUL, ASCII controls, quotes, or semicolons are not accepted.
    InvalidCharacter,
    /// A builder key or value contains a backslash delimiter.
    EmbeddedDelimiter,
}

impl fmt::Display for InfoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TooLong => "info string exceeds its length budget",
            Self::MissingKey => "info string contains an empty key",
            Self::MissingValue => "info key has no value delimiter",
            Self::InvalidCharacter => "info string contains a forbidden character",
            Self::EmbeddedDelimiter => "info field contains an embedded delimiter",
        };
        formatter.write_str(message)
    }
}

impl Error for InfoError {}

pub(super) fn validate_field(bytes: &[u8]) -> Result<(), InfoError> {
    for byte in bytes {
        if matches!(byte, 0..=31 | 127 | b'"' | b';') {
            return Err(InfoError::InvalidCharacter);
        }
        if *byte == b'\\' {
            return Err(InfoError::EmbeddedDelimiter);
        }
    }
    Ok(())
}
