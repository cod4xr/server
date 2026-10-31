// SPDX-License-Identifier: AGPL-3.0-or-later
use std::iter::FusedIterator;

/// A borrowed validated info key and value. Bytes are not assumed to be UTF-8.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InfoPair<'info> {
    pub(super) key: &'info [u8],
    pub(super) value: &'info [u8],
}

impl<'info> InfoPair<'info> {
    /// Returns the key, excluding delimiters.
    #[must_use]
    pub fn key(&self) -> &'info [u8] {
        self.key
    }

    /// Returns the value, excluding delimiters.
    #[must_use]
    pub fn value(&self) -> &'info [u8] {
        self.value
    }
}

/// An allocation-free iterator over an already validated info string.
#[derive(Clone, Debug)]
pub struct InfoPairs<'info> {
    pub(super) remaining: &'info [u8],
}

impl<'info> Iterator for InfoPairs<'info> {
    type Item = InfoPair<'info>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let bytes = self.remaining.strip_prefix(b"\\").unwrap_or(self.remaining);
        let key_end = bytes.iter().position(|&byte| byte == b'\\')?;
        let key = &bytes[..key_end];
        let rest = &bytes[key_end + 1..];
        let value_end = rest
            .iter()
            .position(|&byte| byte == b'\\')
            .unwrap_or(rest.len());
        let value = &rest[..value_end];
        self.remaining = &rest[value_end..];
        Some(InfoPair { key, value })
    }
}

impl FusedIterator for InfoPairs<'_> {}
