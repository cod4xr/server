// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{MessageError, MessageReader, MessageStringLimit};

impl MessageReader<'_> {
    /// Reads a NUL-terminated legacy string, replacing `%` with `.`.
    ///
    /// Output is raw bytes, without the terminator or any UTF-8 assumption. Both
    /// cursors and output remain unchanged on failure. A valid maximum-length
    /// string consumes its terminator; upstream can leave that final NUL unread
    /// when its destination buffer is exactly full.
    ///
    /// # Errors
    /// Rejects missing terminators, profile overflow, and insufficient output.
    pub fn read_legacy_string(
        &mut self,
        output: &mut [u8],
        limit: MessageStringLimit,
    ) -> Result<usize, MessageError> {
        let mut candidate = self.clone();
        let mut length = 0;
        loop {
            if candidate.read_u8()? == 0 {
                break;
            }
            length += 1;
            if length > limit.maximum_bytes() {
                return Err(MessageError::StringTooLong {
                    length,
                    maximum: limit.maximum_bytes(),
                });
            }
        }
        if output.len() < length {
            return Err(MessageError::Capacity {
                needed: length,
                remaining: output.len(),
            });
        }
        let mut copying = self.clone();
        copying.read_bytes(&mut output[..length])?;
        for byte in &mut output[..length] {
            if *byte == b'%' {
                *byte = b'.';
            }
        }
        *self = candidate;
        Ok(length)
    }
}
