// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::zone::{ZonePointerError, ZonePrefixError};

/// Exhausted logical resource in the raw-file-only decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawFileResource {
    /// One name including NUL.
    NameBytes,
    /// Sum of newly read names including NULs.
    TotalNameBytes,
    /// One buffer including final NUL.
    BufferBytes,
    /// Sum of newly read buffers including NULs.
    TotalBufferBytes,
    /// Logical owned index/binding/span storage.
    MetadataBytes,
}

/// Sanitized collection failure; all partial decode state remains private.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RawFileError {
    /// Root prefix failed validation.
    Prefix(ZonePrefixError),
    /// Encoded pointer or declared span is invalid.
    Pointer(ZonePointerError),
    /// A root asset type is outside the supported raw-file-only subset.
    UnsupportedAsset,
    /// An alias names a non-recorded/non-bound pointer slot.
    UnknownAlias,
    /// A name offset has no validated loaded string region.
    UnknownName,
    /// Name is null or uses a sentinel not permitted for this field.
    InvalidNamePointer,
    /// Signed raw-file length is negative.
    InvalidLength,
    /// A nonzero declared length has an absent buffer.
    MissingBuffer,
    /// A complete raw buffer lacks its required final NUL.
    MissingTerminator,
    /// Serialized body/name/buffer is incomplete.
    Truncated,
    /// Temporary or virtual allocation exceeds the declared stream block.
    BlockBounds,
    /// Bytes remain after all supported raw files; no bodies are silently skipped.
    TrailingData,
    /// A configured logical resource limit was exceeded.
    BudgetExceeded(RawFileResource),
    /// Checked arithmetic or host representation failed.
    SizeOverflow,
    /// Fallible staging or index allocation failed.
    AllocationFailed,
}

impl fmt::Display for RawFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "raw-file collection decode failed: {self:?}")
    }
}

impl Error for RawFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Prefix(error) => Some(error),
            Self::Pointer(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ZonePrefixError> for RawFileError {
    fn from(error: ZonePrefixError) -> Self {
        match error {
            ZonePrefixError::Truncated => Self::Truncated,
            error => Self::Prefix(error),
        }
    }
}

impl From<ZonePointerError> for RawFileError {
    fn from(error: ZonePointerError) -> Self {
        Self::Pointer(error)
    }
}

pub(super) fn check(
    resource: RawFileResource,
    size: u64,
    maximum: u64,
) -> Result<(), RawFileError> {
    if size > maximum {
        return Err(RawFileError::BudgetExceeded(resource));
    }
    Ok(())
}
