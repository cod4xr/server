// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Maximum logical message length, matching upstream `MAX_MSGLEN` (128 KiB).
///
/// This is a reassembled message limit, not the size of a UDP datagram.
/// It constrains the current compatibility codec, never engine asset storage,
/// script memory, entity allocation, or a future negotiated extended protocol.
pub const MAX_MESSAGE_BYTES: usize = 0x20000;

/// A rejected message operation. Errors never contain packet contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MessageError {
    /// A string exceeds the selected peer field's byte budget.
    StringTooLong {
        /// Bytes before its terminator.
        length: usize,
        /// Largest string accepted by this field.
        maximum: usize,
    },
    /// A writer input contains an embedded NUL instead of an explicit byte string.
    EmbeddedTerminator,
    /// A bit field width exceeds the 32-bit primitive limit.
    InvalidBitWidth {
        /// Requested number of bits.
        width: u8,
    },
    /// A read requires more new bytes than remain in the message.
    Truncated {
        /// Number of new bytes needed for this operation.
        needed: usize,
        /// Number of unconsumed bytes available.
        remaining: usize,
    },
    /// A write requires more new bytes than the caller's buffer can hold.
    Capacity {
        /// Number of new bytes needed for this operation.
        needed: usize,
        /// Number of unused bytes in the buffer.
        remaining: usize,
    },
    /// A message or backing buffer exceeds the protocol limit.
    MessageTooLarge {
        /// Supplied size; saturated at `usize::MAX` if adding split sizes overflowed.
        length: usize,
        /// Largest accepted size.
        maximum: usize,
    },
}

impl fmt::Display for MessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StringTooLong { length, maximum } => {
                write!(formatter, "string length {length} exceeds {maximum}")
            }
            Self::EmbeddedTerminator => {
                formatter.write_str("string contains an embedded terminator")
            }
            Self::InvalidBitWidth { width } => {
                write!(formatter, "bit width {width} exceeds 32")
            }
            Self::Truncated { needed, remaining } => {
                write!(formatter, "read needs {needed} bytes; {remaining} remain")
            }
            Self::Capacity { needed, remaining } => {
                write!(formatter, "write needs {needed} bytes; {remaining} remain")
            }
            Self::MessageTooLarge { length, maximum } => {
                write!(formatter, "message length {length} exceeds {maximum}")
            }
        }
    }
}

impl Error for MessageError {}

pub(crate) fn bit_bytes_needed(bit_position: usize, width: u8) -> Result<usize, MessageError> {
    if width > 32 {
        return Err(MessageError::InvalidBitWidth { width });
    }
    let cached_bits = (8 - bit_position % 8) % 8;
    Ok(usize::from(width).saturating_sub(cached_bits).div_ceil(8))
}
