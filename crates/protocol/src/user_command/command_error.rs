// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::MessageError;

/// Rejected player-command decoding without cursor or baseline mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UserCommandError {
    /// Caller-supplied baseline lies outside the legacy field/angle domains.
    InvalidBaseline,
    /// The message cannot supply a complete command field.
    Message(MessageError),
}

impl fmt::Display for UserCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBaseline => formatter.write_str("invalid legacy user-command baseline"),
            Self::Message(error) => write!(formatter, "invalid legacy user command: {error}"),
        }
    }
}

impl Error for UserCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidBaseline => None,
            Self::Message(error) => Some(error),
        }
    }
}

impl From<MessageError> for UserCommandError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}
