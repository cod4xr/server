// SPDX-License-Identifier: AGPL-3.0-or-later
// Wire semantics derived from CoD4X src/msg.c; see NOTICE and upstream.lock.json.
use crate::message_error;
use crate::{MAX_MESSAGE_BYTES, MessageError, MessageStringLimit};

/// Writes bounded `CoD4X` messages into caller-owned storage without allocation.
///
/// Only [`Self::as_bytes`] exposes initialized message bytes. Unused bits in
/// newly allocated bit bytes are zeroed, even if the backing storage is dirty.
#[derive(Debug)]
pub struct MessageWriter<'buffer> {
    storage: &'buffer mut [u8],
    length: usize,
    bit_position: usize,
}

impl<'buffer> MessageWriter<'buffer> {
    /// Creates an empty writer using all of `storage` as its capacity.
    ///
    /// # Errors
    /// Returns [`MessageError::MessageTooLarge`] if storage exceeds 128 KiB.
    pub fn try_new(storage: &'buffer mut [u8]) -> Result<Self, MessageError> {
        if storage.len() > MAX_MESSAGE_BYTES {
            return Err(MessageError::MessageTooLarge {
                length: storage.len(),
                maximum: MAX_MESSAGE_BYTES,
            });
        }
        Ok(Self {
            storage,
            length: 0,
            bit_position: 0,
        })
    }

    /// Returns the encoded message, including any partially filled bit byte.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.storage[..self.length]
    }

    /// Returns the number of initialized message bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns whether no bytes have been written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns the number of new bytes the writer can append.
    ///
    /// A full buffer may still accept bits in an existing partially filled byte.
    #[must_use]
    pub fn remaining_capacity(&self) -> usize {
        self.storage.len() - self.length
    }

    /// Resets both cursors for reuse. This does not securely erase the buffer.
    pub fn clear(&mut self) {
        self.length = 0;
        self.bit_position = 0;
    }

    /// Appends an unsigned byte without moving the bit cursor.
    ///
    /// # Errors
    /// Returns [`MessageError::Capacity`] if no new byte fits.
    pub fn write_u8(&mut self, value: u8) -> Result<(), MessageError> {
        self.write_bytes(&[value])
    }

    /// Appends a signed 16-bit integer in little-endian order.
    ///
    /// # Errors
    /// Returns [`MessageError::Capacity`] if two new bytes do not fit.
    pub fn write_i16(&mut self, value: i16) -> Result<(), MessageError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Appends a signed 32-bit integer in little-endian order.
    ///
    /// # Errors
    /// Returns [`MessageError::Capacity`] if four new bytes do not fit.
    pub fn write_i32(&mut self, value: i32) -> Result<(), MessageError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Appends a signed 64-bit integer in little-endian order.
    ///
    /// # Errors
    /// Returns [`MessageError::Capacity`] if eight new bytes do not fit.
    pub fn write_i64(&mut self, value: i64) -> Result<(), MessageError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Appends an IEEE 754 binary32 value, preserving its exact bit pattern.
    ///
    /// # Errors
    /// Returns [`MessageError::Capacity`] if four new bytes do not fit.
    pub fn write_f32(&mut self, value: f32) -> Result<(), MessageError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Appends raw bytes without moving the bit cursor.
    ///
    /// # Errors
    /// Returns [`MessageError::Capacity`] if the entire slice does not fit.
    /// No bytes are written on failure.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), MessageError> {
        self.ensure_capacity(bytes.len())?;
        let end = self.length + bytes.len();
        self.storage[self.length..end].copy_from_slice(bytes);
        self.length = end;
        Ok(())
    }

    /// Writes the lowest `width` bits, least-significant bit first.
    ///
    /// Bits reuse an earlier partial bit byte even after intervening byte writes.
    /// Width zero is a no-op; bits above `width` are ignored, matching upstream.
    ///
    /// # Errors
    /// Returns [`MessageError::InvalidBitWidth`] for widths above 32, or
    /// [`MessageError::Capacity`] if the full field does not fit.
    pub fn write_bits(&mut self, value: u32, width: u8) -> Result<(), MessageError> {
        let needed = message_error::bit_bytes_needed(self.bit_position, width)?;
        self.ensure_capacity(needed)?;
        for shift in 0..width {
            if self.bit_position.is_multiple_of(8) {
                self.bit_position = self.length * 8;
                self.storage[self.length] = 0;
                self.length += 1;
            }
            if (value >> shift) & 1 != 0 {
                self.storage[self.bit_position / 8] |= 1 << (self.bit_position % 8);
            }
            self.bit_position += 1;
        }
        Ok(())
    }

    /// Appends a legacy byte string and its NUL terminator.
    ///
    /// The input excludes its terminator. Existing cached bits remain available
    /// for subsequent bit writes, as with other byte operations.
    ///
    /// # Errors
    /// Rejects embedded NUL, profile overflow, and insufficient output atomically.
    pub fn write_legacy_string(
        &mut self,
        bytes: &[u8],
        limit: MessageStringLimit,
    ) -> Result<(), MessageError> {
        if bytes.len() > limit.maximum_bytes() {
            return Err(MessageError::StringTooLong {
                length: bytes.len(),
                maximum: limit.maximum_bytes(),
            });
        }
        if bytes.contains(&0) {
            return Err(MessageError::EmbeddedTerminator);
        }
        self.ensure_capacity(bytes.len() + 1)?;
        let end = self.length + bytes.len();
        self.storage[self.length..end].copy_from_slice(bytes);
        self.storage[end] = 0;
        self.length = end + 1;
        Ok(())
    }

    fn ensure_capacity(&self, needed: usize) -> Result<(), MessageError> {
        let remaining = self.remaining_capacity();
        if needed > remaining {
            return Err(MessageError::Capacity { needed, remaining });
        }
        Ok(())
    }
}
