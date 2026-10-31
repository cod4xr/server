// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use cod4x_protocol::PacketError;

/// A channel setup or transmit failure, without message contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ChannelError {
    /// Supplied storage or an outbound logical message exceeds the protocol limit.
    MessageTooLarge,
    /// The channel must reconnect before reserved marker/sequence ranges are reached.
    SequenceExhausted,
    /// The complete next packet cannot fit or its envelope cannot be represented.
    Packet(PacketError),
}

impl fmt::Display for ChannelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MessageTooLarge => formatter.write_str("channel message exceeds its size budget"),
            Self::SequenceExhausted => formatter.write_str("channel sequence requires reconnect"),
            Self::Packet(error) => write!(formatter, "channel packet: {error}"),
        }
    }
}

impl Error for ChannelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Packet(error) => Some(error),
            Self::MessageTooLarge | Self::SequenceExhausted => None,
        }
    }
}

impl From<PacketError> for ChannelError {
    fn from(error: PacketError) -> Self {
        Self::Packet(error)
    }
}
