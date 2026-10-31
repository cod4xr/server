// SPDX-License-Identifier: AGPL-3.0-or-later
use super::CommandTokens;
use crate::MessageError;

/// Maximum command-line bytes, excluding the original C-string terminator.
pub const MAX_COMMAND_BYTES: usize = 1023;

/// A bounded command line ready for allocation-free tokenization.
#[derive(Clone, Copy, Debug)]
pub struct CommandLine<'line> {
    bytes: &'line [u8],
    consumed_bytes: usize,
}

impl<'line> CommandLine<'line> {
    /// Borrows one raw command line, stopping at its first NUL or newline.
    ///
    /// This preserves percent signs, as direct command tokenization does.
    ///
    /// # Errors
    /// Returns [`MessageError::MessageTooLarge`] for lines above 1023 bytes.
    pub fn parse(bytes: &'line [u8]) -> Result<Self, MessageError> {
        let length = bytes
            .iter()
            .take(MAX_COMMAND_BYTES + 1)
            .position(|&byte| byte == 0 || byte == b'\n')
            .unwrap_or(bytes.len());
        if length > MAX_COMMAND_BYTES {
            return Err(MessageError::MessageTooLarge {
                length,
                maximum: MAX_COMMAND_BYTES,
            });
        }
        Ok(Self {
            bytes: &bytes[..length],
            consumed_bytes: (length + 1).min(bytes.len()),
        })
    }

    /// Copies an out-of-band command line, replacing `%` with `.` as upstream
    /// `MSG_ReadStringLine` does before tokenizing the result.
    ///
    /// Newline/NUL delimiters count toward [`Self::consumed_bytes`] but are not
    /// copied. Binary bytes following the delimiter remain uninterpreted.
    ///
    /// # Errors
    /// Rejects oversized lines and insufficient output without changing output.
    pub fn read_connectionless(body: &[u8], output: &'line mut [u8]) -> Result<Self, MessageError> {
        let source = CommandLine::parse(body)?;
        let length = source.bytes.len();
        if output.len() < length {
            return Err(MessageError::Capacity {
                needed: length,
                remaining: output.len(),
            });
        }
        for (target, &byte) in output[..length].iter_mut().zip(source.bytes) {
            *target = if byte == b'%' { b'.' } else { byte };
        }
        Ok(Self {
            bytes: &output[..length],
            consumed_bytes: source.consumed_bytes,
        })
    }

    /// Returns the bounded line without its delimiter or trailing body bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &'line [u8] {
        self.bytes
    }

    /// Returns the source bytes consumed, including a newline/NUL if present.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }

    /// Returns tokens without allocating, unescaping, or requiring UTF-8.
    #[must_use]
    pub fn tokens(&self) -> CommandTokens<'line> {
        CommandTokens {
            remaining: self.bytes,
        }
    }
}
