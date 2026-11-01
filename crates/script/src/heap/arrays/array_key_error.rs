// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::heap::HeapErrorKind;

/// Sanitized legacy key conversion error without key bytes, indices or handles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyArrayKeyError {
    /// The source value is not a strict integer or a plain string.
    ExpectedIndex,
    /// The integer is outside the named native script-index compatibility domain.
    IndexOutOfRange,
    /// Checked plain-string storage rejected the supplied handle.
    StringHeap(HeapErrorKind),
}

impl fmt::Display for LegacyArrayKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ExpectedIndex => "legacy array key requires an integer or plain string",
            Self::IndexOutOfRange => "integer array key is outside the legacy script profile",
            Self::StringHeap(_) => "legacy array key string reference is invalid",
        })
    }
}

impl Error for LegacyArrayKeyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StringHeap(error) => Some(error),
            Self::ExpectedIndex | Self::IndexOutOfRange => None,
        }
    }
}
