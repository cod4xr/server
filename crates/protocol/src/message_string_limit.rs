// SPDX-License-Identifier: AGPL-3.0-or-later
/// Legacy wire string profiles, excluding their required NUL terminator.
/// These do not constrain engine script strings, content names, or asset storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageStringLimit {
    /// Original `MAX_STRING_CHARS` message fields.
    Normal,
    /// `BIG_INFO_STRING` message fields.
    Big,
}

impl MessageStringLimit {
    /// Returns the maximum number of bytes before the NUL terminator.
    #[must_use]
    pub fn maximum_bytes(self) -> usize {
        match self {
            Self::Normal => 1023,
            Self::Big => 8191,
        }
    }
}
