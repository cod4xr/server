// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Borrowed validated name/buffer of one serialized raw file, without registration.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RawFile<'payload> {
    pub(super) name: &'payload [u8],
    pub(super) contents: Option<&'payload [u8]>,
}

impl<'payload> RawFile<'payload> {
    /// Returns exact name bytes excluding their validated NUL, without path repair.
    #[must_use]
    pub fn name(self) -> &'payload [u8] {
        self.name
    }

    /// Returns bytes excluding the validated final NUL, or an encoded null buffer.
    ///
    /// Binary contents/interior NULs are retained. A present empty buffer differs
    /// from an absent zero-length buffer. This does not parse or execute scripts.
    #[must_use]
    pub fn contents(self) -> Option<&'payload [u8]> {
        self.contents
    }
}

impl fmt::Debug for RawFile<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawFile")
            .field("name_bytes", &self.name.len())
            .field("buffer_bytes", &self.contents.map(<[u8]>::len))
            .finish_non_exhaustive()
    }
}
