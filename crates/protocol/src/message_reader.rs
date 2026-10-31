// SPDX-License-Identifier: AGPL-3.0-or-later
// Wire semantics derived from CoD4X src/msg.c; see NOTICE and upstream.lock.json.
use crate::message_error;
use crate::{MAX_MESSAGE_BYTES, MessageError};

/// Reads a bounded `CoD4X` message from one or two borrowed slices.
///
/// The split boundary is a storage detail and does not affect decoding. A
/// partially consumed bit byte remains available after intervening byte reads.
#[derive(Clone, Debug)]
pub struct MessageReader<'message> {
    first: &'message [u8],
    second: &'message [u8],
    consumed_bytes: usize,
    bit_position: usize,
}

impl<'message> MessageReader<'message> {
    /// Creates a reader for a contiguous message.
    ///
    /// # Errors
    /// Returns [`MessageError::MessageTooLarge`] for messages above 128 KiB.
    pub fn try_new(bytes: &'message [u8]) -> Result<Self, MessageError> {
        Self::try_new_split(bytes, &[])
    }

    /// Creates a reader for the logical concatenation of two slices.
    ///
    /// # Errors
    /// Returns [`MessageError::MessageTooLarge`] if their total exceeds 128 KiB.
    pub fn try_new_split(
        first: &'message [u8],
        second: &'message [u8],
    ) -> Result<Self, MessageError> {
        let length = first.len().saturating_add(second.len());
        if length > MAX_MESSAGE_BYTES {
            return Err(MessageError::MessageTooLarge {
                length,
                maximum: MAX_MESSAGE_BYTES,
            });
        }
        Ok(Self {
            first,
            second,
            consumed_bytes: 0,
            bit_position: 0,
        })
    }

    /// Returns the number of unconsumed whole bytes.
    ///
    /// Zero does not imply there are no cached bits remaining in a partial byte.
    #[must_use]
    pub fn remaining_bytes(&self) -> usize {
        self.first.len() + self.second.len() - self.consumed_bytes
    }

    /// Resets both cursors to the beginning of the message.
    pub fn rewind(&mut self) {
        self.consumed_bytes = 0;
        self.bit_position = 0;
    }

    /// Reads an unsigned byte without moving the bit cursor.
    ///
    /// # Errors
    /// Returns [`MessageError::Truncated`] if no new byte remains.
    pub fn read_u8(&mut self) -> Result<u8, MessageError> {
        Ok(u8::from_le_bytes(self.read_array()?))
    }

    /// Reads a signed little-endian 16-bit integer.
    ///
    /// # Errors
    /// Returns [`MessageError::Truncated`] if two new bytes do not remain.
    pub fn read_i16(&mut self) -> Result<i16, MessageError> {
        Ok(i16::from_le_bytes(self.read_array()?))
    }

    /// Reads a signed little-endian 32-bit integer.
    ///
    /// # Errors
    /// Returns [`MessageError::Truncated`] if four new bytes do not remain.
    pub fn read_i32(&mut self) -> Result<i32, MessageError> {
        Ok(i32::from_le_bytes(self.read_array()?))
    }

    /// Reads a signed little-endian 64-bit integer.
    ///
    /// # Errors
    /// Returns [`MessageError::Truncated`] if eight new bytes do not remain.
    pub fn read_i64(&mut self) -> Result<i64, MessageError> {
        Ok(i64::from_le_bytes(self.read_array()?))
    }

    /// Reads an IEEE 754 binary32 value, preserving its exact bit pattern.
    ///
    /// # Errors
    /// Returns [`MessageError::Truncated`] if four new bytes do not remain.
    pub fn read_f32(&mut self) -> Result<f32, MessageError> {
        Ok(f32::from_le_bytes(self.read_array()?))
    }

    /// Copies bytes across either backing slice without moving the bit cursor.
    ///
    /// # Errors
    /// Returns [`MessageError::Truncated`] if the entire output cannot be filled.
    /// The output and both cursors are unchanged on failure.
    pub fn read_bytes(&mut self, output: &mut [u8]) -> Result<(), MessageError> {
        self.ensure_available(output.len())?;
        for (offset, byte) in output.iter_mut().enumerate() {
            *byte = self.byte_at(self.consumed_bytes + offset);
        }
        self.consumed_bytes += output.len();
        Ok(())
    }

    /// Reads a field of up to 32 bits, least-significant bit first.
    ///
    /// Width zero returns zero without consuming input. All 32 bits are returned
    /// without signed interpretation; signed protocol fields decode separately.
    ///
    /// # Errors
    /// Returns [`MessageError::InvalidBitWidth`] for widths above 32, or
    /// [`MessageError::Truncated`] if the complete field is unavailable.
    pub fn read_bits(&mut self, width: u8) -> Result<u32, MessageError> {
        let needed = message_error::bit_bytes_needed(self.bit_position, width)?;
        self.ensure_available(needed)?;
        let mut value = 0_u32;
        for shift in 0..width {
            if self.bit_position.is_multiple_of(8) {
                self.bit_position = self.consumed_bytes * 8;
                self.consumed_bytes += 1;
            }
            let byte = self.byte_at(self.bit_position / 8);
            let bit = (byte >> (self.bit_position % 8)) & 1;
            value |= u32::from(bit) << shift;
            self.bit_position += 1;
        }
        Ok(value)
    }

    fn read_array<const LENGTH: usize>(&mut self) -> Result<[u8; LENGTH], MessageError> {
        let mut bytes = [0; LENGTH];
        self.read_bytes(&mut bytes)?;
        Ok(bytes)
    }

    fn ensure_available(&self, needed: usize) -> Result<(), MessageError> {
        let remaining = self.remaining_bytes();
        if needed > remaining {
            return Err(MessageError::Truncated { needed, remaining });
        }
        Ok(())
    }

    // Every caller preflights its full read before indexing either slice.
    fn byte_at(&self, offset: usize) -> u8 {
        if offset < self.first.len() {
            self.first[offset]
        } else {
            self.second[offset - self.first.len()]
        }
    }
}
