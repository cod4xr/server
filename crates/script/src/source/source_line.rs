// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Recognized line ending; standalone CR remains ordinary source content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEnding {
    /// Final source line with no LF terminator.
    None,
    /// One LF byte.
    Lf,
    /// A CR immediately followed by LF.
    CrLf,
}

/// Borrowed line bytes preserving source offset, contents, and exact ending.
#[derive(Clone, Copy)]
pub struct SourceLine<'source> {
    offset: u64,
    bytes: &'source [u8],
    ending: LineEnding,
}

impl<'source> SourceLine<'source> {
    /// Returns this line's zero-based start offset within its source.
    #[must_use]
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Returns all exact bytes including any CRLF/LF ending.
    #[must_use]
    pub fn full_bytes(self) -> &'source [u8] {
        self.bytes
    }

    /// Returns exact content bytes excluding only the recognized ending.
    #[must_use]
    pub fn content(self) -> &'source [u8] {
        let suffix = match self.ending {
            LineEnding::None => 0,
            LineEnding::Lf => 1,
            LineEnding::CrLf => 2,
        };
        &self.bytes[..self.bytes.len() - suffix]
    }

    /// Returns the ending classification without normalizing source bytes.
    #[must_use]
    pub fn ending(self) -> LineEnding {
        self.ending
    }

    pub(super) fn new(offset: u64, bytes: &'source [u8]) -> Self {
        let ending = if bytes.ends_with(b"\r\n") {
            LineEnding::CrLf
        } else if bytes.ends_with(b"\n") {
            LineEnding::Lf
        } else {
            LineEnding::None
        };
        Self {
            offset,
            bytes,
            ending,
        }
    }
}

impl fmt::Debug for SourceLine<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceLine")
            .field("offset", &self.offset)
            .field("bytes", &self.bytes.len())
            .field("ending", &self.ending)
            .finish_non_exhaustive()
    }
}
