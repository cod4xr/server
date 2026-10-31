// SPDX-License-Identifier: AGPL-3.0-or-later
// Ordering and exact-key replacement follow upstream qshared.c; see NOTICE.
use super::info_error;
use super::{InfoError, InfoLimit};

/// A bounded info-string builder using caller-owned storage.
///
/// New values prepend their key/value pair. Replacing/removing a key removes its
/// first exact-case match, matching upstream `Info_SetValueForKey` semantics.
#[derive(Debug)]
pub struct InfoWriter<'buffer> {
    storage: &'buffer mut [u8],
    length: usize,
    limit: InfoLimit,
}

impl<'buffer> InfoWriter<'buffer> {
    /// Creates an empty builder; its limit includes the caller's storage budget.
    #[must_use]
    pub fn new(storage: &'buffer mut [u8], limit: InfoLimit) -> Self {
        Self {
            storage,
            length: 0,
            limit,
        }
    }

    /// Returns encoded bytes without a C-string terminator.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.storage[..self.length]
    }

    /// Inserts or replaces a key; an empty value removes that exact-case key.
    ///
    /// Every error leaves the entire encoded string unchanged. This deliberately
    /// avoids upstream's remove-before-capacity-check loss of existing values.
    ///
    /// # Errors
    /// Rejects empty keys, invalid characters, embedded delimiters, and writes
    /// exceeding the selected profile or available caller storage.
    pub fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), InfoError> {
        if key.is_empty() {
            return Err(InfoError::MissingKey);
        }
        if key.len() > self.limit.maximum_bytes() || value.len() > self.limit.maximum_bytes() {
            return Err(InfoError::TooLong);
        }
        info_error::validate_field(key)?;
        info_error::validate_field(value)?;
        let removed = self.exact_pair_range(key);
        let removed_length = removed.as_ref().map_or(0, ExactSizeIterator::len);
        let added_length = if value.is_empty() {
            0
        } else {
            key.len().saturating_add(value.len()).saturating_add(2)
        };
        let retained_length = self.length - removed_length;
        let capacity = self.storage.len().min(self.limit.maximum_bytes());
        if added_length > capacity - retained_length {
            return Err(InfoError::TooLong);
        }
        if let Some(range) = removed {
            self.storage
                .copy_within(range.end..self.length, range.start);
        }
        if added_length != 0 {
            self.storage.copy_within(..retained_length, added_length);
            self.storage[0] = b'\\';
            self.storage[1..=key.len()].copy_from_slice(key);
            self.storage[key.len() + 1] = b'\\';
            self.storage[key.len() + 2..added_length].copy_from_slice(value);
        }
        self.length = retained_length + added_length;
        Ok(())
    }

    fn exact_pair_range(&self, key: &[u8]) -> Option<std::ops::Range<usize>> {
        let mut start = 0;
        let bytes = self.as_bytes();
        // The builder only creates canonical, validated pairs with leading slashes.
        while start < bytes.len() {
            let key_start = start + 1;
            let key_length = bytes[key_start..].iter().position(|&byte| byte == b'\\')?;
            let value_start = key_start + key_length + 1;
            let value_length = bytes[value_start..]
                .iter()
                .position(|&byte| byte == b'\\')
                .unwrap_or(bytes.len() - value_start);
            let end = value_start + value_length;
            if &bytes[key_start..key_start + key_length] == key {
                return Some(start..end);
            }
            start = end;
        }
        None
    }
}
