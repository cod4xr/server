// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Fixed identifier admission failure; no received spelling or host path is stored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyIdentifierError {
    /// The lexeme is not an original identifier terminal.
    WrongToken,
    /// This explicit C-locale ASCII profile does not admit non-ASCII names.
    NonAscii,
}

impl fmt::Display for LegacyIdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongToken => formatter.write_str("expected legacy identifier token"),
            Self::NonAscii => formatter.write_str("legacy identifier profile requires ASCII"),
        }
    }
}

impl Error for LegacyIdentifierError {}
