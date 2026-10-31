// SPDX-License-Identifier: AGPL-3.0-or-later
//! Safe, allocation-free primitives for the `CoD4X` message format.
//!
//! Byte operations and bit operations use separate cursors. Byte operations
//! append or consume entire bytes; later bit operations can still use the
//! unused bits of an earlier byte. This is intentionally different from a
//! conventional contiguous bitstream.
//!
//! Every fallible operation is transactional: failure leaves cursors, output,
//! and written bytes unchanged. The caller must propagate errors and discard
//! malformed messages rather than treating errors as zero-valued fields.
//!
//! ```
//! use cod4x_protocol::{MessageReader, MessageWriter};
//!
//! # fn main() -> Result<(), cod4x_protocol::MessageError> {
//! let mut storage = [0; 32];
//! let mut writer = MessageWriter::try_new(&mut storage)?;
//! writer.write_bits(5, 3)?;
//! writer.write_u8(0xaa)?;
//! writer.write_bits(27, 5)?;
//! assert_eq!(writer.as_bytes(), &[0xdd, 0xaa]);
//!
//! let mut reader = MessageReader::try_new(writer.as_bytes())?;
//! assert_eq!(reader.read_bits(3)?, 5);
//! assert_eq!(reader.read_u8()?, 0xaa);
//! assert_eq!(reader.read_bits(5)?, 27);
//! # Ok(())
//! # }
//! ```

mod command;
pub mod compression;
mod index;
mod info;
mod message_error;
mod message_reader;
mod message_string_limit;
mod message_string_reader;
mod message_writer;
pub mod obfuscation;
mod packet;

pub use command::{CommandLine, CommandTokens, MAX_COMMAND_BYTES};
pub use index::{LegacyIndexDecoder, LegacyIndexEncoder, LegacyIndexError, LegacyIndexKind};
pub use info::{InfoError, InfoLimit, InfoPair, InfoPairs, InfoString, InfoWriter};
pub use message_error::{MAX_MESSAGE_BYTES, MessageError};
pub use message_reader::MessageReader;
pub use message_string_limit::MessageStringLimit;
pub use message_writer::MessageWriter;
pub use packet::{
    ChannelPacket, Direction, FRAGMENT_BYTES, MAX_PACKET_BYTES, Packet, PacketError, Route,
};
