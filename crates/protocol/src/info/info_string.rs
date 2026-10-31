// SPDX-License-Identifier: AGPL-3.0-or-later
use super::info_error;
use super::{InfoError, InfoPairs};

/// Length profile for an info string, excluding its C-string terminator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfoLimit {
    /// `MAX_INFO_STRING` is 1024 bytes including the terminator.
    Normal,
    /// `BIG_INFO_STRING` is 8192 bytes including the terminator.
    Big,
}

impl InfoLimit {
    /// Returns the maximum encoded byte length without a terminator.
    #[must_use]
    pub fn maximum_bytes(self) -> usize {
        match self {
            Self::Normal => 1023,
            Self::Big => 8191,
        }
    }
}

/// A fully validated borrowed key/value string in the Quake-derived wire format.
///
/// Duplicate keys and optional initial backslashes are preserved. Lookup returns
/// the first ASCII-case-insensitive match, following upstream `Info_ValueForKey`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InfoString<'info> {
    bytes: &'info [u8],
}

impl<'info> InfoString<'info> {
    /// Validates the complete string before any caller can use its fields.
    ///
    /// Empty values are valid; empty keys, dangling delimiters, ASCII controls,
    /// embedded terminators, quotes, and semicolons are rejected. High bytes are
    /// retained verbatim instead of being forced into UTF-8.
    ///
    /// # Errors
    /// Returns an [`InfoError`] describing structural or resource-limit failures.
    pub fn parse(bytes: &'info [u8], limit: InfoLimit) -> Result<Self, InfoError> {
        if bytes.len() > limit.maximum_bytes() {
            return Err(InfoError::TooLong);
        }
        let mut remaining = bytes;
        while !remaining.is_empty() {
            let fields = remaining.strip_prefix(b"\\").unwrap_or(remaining);
            let key_end = fields
                .iter()
                .position(|&byte| byte == b'\\')
                .ok_or(InfoError::MissingValue)?;
            let key = &fields[..key_end];
            if key.is_empty() {
                return Err(InfoError::MissingKey);
            }
            info_error::validate_field(key)?;
            let rest = &fields[key_end + 1..];
            let value_end = rest
                .iter()
                .position(|&byte| byte == b'\\')
                .unwrap_or(rest.len());
            info_error::validate_field(&rest[..value_end])?;
            remaining = &rest[value_end..];
        }
        Ok(Self { bytes })
    }

    /// Returns the original encoded bytes without a C-string terminator.
    #[must_use]
    pub fn as_bytes(&self) -> &'info [u8] {
        self.bytes
    }

    /// Returns fields in their original wire order, including duplicate keys.
    #[must_use]
    pub fn pairs(&self) -> InfoPairs<'info> {
        InfoPairs {
            remaining: self.bytes,
        }
    }

    /// Returns the first case-insensitive matching value, or `None` if absent.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&'info [u8]> {
        self.pairs()
            .find(|pair| pair.key().eq_ignore_ascii_case(key))
            .map(|pair| pair.value())
    }
}
