// SPDX-License-Identifier: AGPL-3.0-or-later
// Envelope behavior follows upstream netchan.c and sv_main.c; see NOTICE.
use super::{ChannelPacket, Direction, Packet, PacketError, Route};
use crate::{MAX_MESSAGE_BYTES, MessageError, MessageReader, MessageWriter};

impl<'packet> Packet<'packet> {
    /// Parses a datagram without allocation or modifying its storage.
    ///
    /// # Errors
    /// Rejects missing fields, invalid fragment bounds, oversized envelopes,
    /// and fragment bodies whose declared lengths differ from their actual size.
    pub fn parse(bytes: &'packet [u8], direction: Direction) -> Result<Self, PacketError> {
        let mut reader = MessageReader::try_new(bytes)?;
        let marker = reader.read_i32()?;
        match marker {
            -1 => return Ok(Self::Connectionless(&bytes[4..])),
            -2 => return Ok(Self::UpdateProxy(&bytes[4..])),
            -16 => {
                let qport = u16::from_le_bytes(reader.read_i16()?.to_le_bytes());
                return Ok(Self::Reliable {
                    qport,
                    body: &bytes[6..],
                });
            }
            _ => {}
        }
        let raw = u32::from_le_bytes(marker.to_le_bytes());
        let route = match direction {
            Direction::ClientToServer => {
                let qport = u16::from_le_bytes(reader.read_i16()?.to_le_bytes());
                Route::ToServer { qport }
            }
            Direction::ServerToClient => Route::ToClient,
        };
        let fragmented = raw & 0x8000_0000 != 0;
        let fragment_start = if fragmented {
            let start =
                u32::try_from(reader.read_i32()?).map_err(|_| PacketError::InvalidFragment)?;
            let length =
                usize::try_from(reader.read_i16()?).map_err(|_| PacketError::InvalidFragment)?;
            if length != reader.remaining_bytes() {
                return Err(PacketError::FragmentLengthMismatch);
            }
            Some(start)
        } else {
            None
        };
        let body = &bytes[bytes.len() - reader.remaining_bytes()..];
        let channel = ChannelPacket::try_new(raw & 0x7fff_ffff, route, fragment_start, body)?;
        Ok(Self::Channel(channel))
    }

    /// Encodes the complete envelope and returns its length.
    ///
    /// On error, output remains unchanged. Caller storage may exceed the logical
    /// message limit; only the validated encoded prefix is used.
    ///
    /// # Errors
    /// Rejects an oversized envelope or insufficient caller storage.
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PacketError> {
        let (header, body) = match self {
            Self::Connectionless(body) | Self::UpdateProxy(body) => (4, *body),
            Self::Reliable { body, .. } => (6, *body),
            Self::Channel(channel) => {
                let route_length = match channel.route() {
                    Route::ToServer { .. } => 6,
                    Route::ToClient => 4,
                };
                let fragment_length = if channel.fragment_start().is_some() {
                    6
                } else {
                    0
                };
                (route_length + fragment_length, channel.body())
            }
        };
        let length = header + body.len();
        if length > MAX_MESSAGE_BYTES {
            return Err(MessageError::MessageTooLarge {
                length,
                maximum: MAX_MESSAGE_BYTES,
            }
            .into());
        }
        if output.len() < length {
            return Err(MessageError::Capacity {
                needed: length,
                remaining: output.len(),
            }
            .into());
        }
        let mut writer = MessageWriter::try_new(&mut output[..length])?;
        match self {
            Self::Connectionless(_) => writer.write_i32(-1)?,
            Self::UpdateProxy(_) => writer.write_i32(-2)?,
            Self::Reliable { qport, .. } => {
                writer.write_i32(-16)?;
                writer.write_bytes(&qport.to_le_bytes())?;
            }
            Self::Channel(channel) => {
                let raw = channel.sequence()
                    | if channel.fragment_start().is_some() {
                        0x8000_0000
                    } else {
                        0
                    };
                writer.write_bytes(&raw.to_le_bytes())?;
                if let Route::ToServer { qport } = channel.route() {
                    writer.write_bytes(&qport.to_le_bytes())?;
                }
                if let Some(start) = channel.fragment_start() {
                    writer.write_bytes(&start.to_le_bytes())?;
                    let length =
                        u16::try_from(body.len()).map_err(|_| PacketError::InvalidFragment)?;
                    writer.write_bytes(&length.to_le_bytes())?;
                }
            }
        }
        writer.write_bytes(body)?;
        Ok(length)
    }
}
