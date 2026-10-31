// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::MessageError;

/// A malformed or unrepresentable datagram envelope, without packet contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PacketError {
    /// A primitive field is truncated or its output does not fit.
    Message(MessageError),
    /// A sequence cannot be represented without colliding with a packet marker.
    InvalidSequence,
    /// A fragment has a negative or out-of-range offset/length.
    InvalidFragment,
    /// Declared fragment length differs from the actual body length.
    FragmentLengthMismatch,
}

impl fmt::Display for PacketError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(error) => write!(formatter, "packet envelope: {error}"),
            Self::InvalidSequence => formatter.write_str("invalid packet sequence"),
            Self::InvalidFragment => formatter.write_str("invalid fragment bounds"),
            Self::FragmentLengthMismatch => formatter.write_str("fragment length mismatch"),
        }
    }
}

impl Error for PacketError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Message(error) => Some(error),
            Self::InvalidSequence | Self::InvalidFragment | Self::FragmentLengthMismatch => None,
        }
    }
}

impl From<MessageError> for PacketError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}
