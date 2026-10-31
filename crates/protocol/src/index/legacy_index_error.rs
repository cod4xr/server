// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::MessageError;

/// A rejected legacy snapshot index operation, without snapshot contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LegacyIndexError {
    /// An index exceeds its selected legacy wire namespace.
    OutOfRange,
    /// Snapshot index streams must strictly increase between resets.
    NotIncreasing,
    /// A primitive read/write cannot complete.
    Message(MessageError),
}

impl fmt::Display for LegacyIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange => formatter.write_str("index exceeds the legacy peer namespace"),
            Self::NotIncreasing => formatter.write_str("snapshot indices must strictly increase"),
            Self::Message(error) => write!(formatter, "snapshot index: {error}"),
        }
    }
}

impl Error for LegacyIndexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Message(error) => Some(error),
            Self::OutOfRange | Self::NotIncreasing => None,
        }
    }
}

impl From<MessageError> for LegacyIndexError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}
