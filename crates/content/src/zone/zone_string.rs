// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// A serialized script-string slot, without an interned engine handle.
///
/// Inline bytes borrow the owned fastfile stream. Encoded references are retained
/// verbatim and are not validated as addresses or followed by prefix inspection.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ZoneString<'payload> {
    /// Encoded null slot; distinct from an inline empty string.
    Null,
    /// Inline bytes excluding their NUL terminator; UTF-8 is not required.
    Inline(&'payload [u8]),
    /// Non-null/non-inline encoded value, requiring a typed block resolver.
    Unresolved(u32),
}

impl fmt::Debug for ZoneString<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("ZoneString::Null"),
            Self::Inline(bytes) => formatter
                .debug_struct("ZoneString::Inline")
                .field("bytes", &bytes.len())
                .finish_non_exhaustive(),
            Self::Unresolved(_) => formatter.write_str("ZoneString::Unresolved(..)"),
        }
    }
}
