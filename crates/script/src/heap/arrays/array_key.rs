// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Borrowed authoritative array key, independent of packed native names/string IDs.
///
/// This type allocates nothing and grants no lookup/mutation/ownership authority.
/// Storage must admit owned copies, hashing and comparisons separately. Raw byte
/// keys preserve complete content; native NUL/interner behavior is not inferred.
#[derive(Clone, Copy)]
pub enum ArrayKey<'a> {
    /// Wide engine integer key, without an inherited native index ceiling.
    Index(u64),
    /// Exact bytes without case folding, UTF-8 conversion or implicit terminator.
    Bytes(&'a [u8]),
}

impl fmt::Debug for ArrayKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(_) => formatter.write_str("ArrayKey::Index(..)"),
            Self::Bytes(bytes) => formatter
                .debug_struct("ArrayKey::Bytes")
                .field("length", &bytes.len())
                .finish(),
        }
    }
}
