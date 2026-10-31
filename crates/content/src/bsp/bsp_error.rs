// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Resource rejected before publishing BSP container metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BspResource {
    /// Complete encoded file length.
    FileBytes,
    /// Directory entry count and associated metadata work/storage.
    Lumps,
    /// One lump's encoded bytes.
    LumpBytes,
    /// Sum of encoded lump lengths, including repeated spans.
    TotalLumpBytes,
}

/// Sanitized container failure without filename, entity text, or payload bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BspError {
    /// The complete header, directory, or payload span is not available.
    Truncated,
    /// The file identifier is not the expected IBSP byte sequence.
    InvalidMagic,
    /// The pinned source has no supported container layout for this version.
    UnsupportedVersion,
    /// A nonempty old-format payload points into its own header/directory.
    DirectoryOverlap,
    /// A configured admission limit was exceeded.
    BudgetExceeded(BspResource),
    /// An offset/length cannot be represented or checked arithmetic overflowed.
    SizeOverflow,
    /// Fallible directory metadata allocation failed.
    AllocationFailed,
    /// Element size is zero or does not divide the complete lump length.
    InvalidElementSize,
}

impl fmt::Display for BspError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => formatter.write_str("truncated BSP container"),
            Self::InvalidMagic => formatter.write_str("invalid BSP identifier"),
            Self::UnsupportedVersion => formatter.write_str("unsupported BSP container version"),
            Self::DirectoryOverlap => formatter.write_str("BSP payload overlaps its directory"),
            Self::BudgetExceeded(resource) => {
                write!(formatter, "BSP budget exceeded: {resource:?}")
            }
            Self::SizeOverflow => formatter.write_str("BSP size cannot be represented"),
            Self::AllocationFailed => formatter.write_str("BSP metadata allocation failed"),
            Self::InvalidElementSize => formatter.write_str("invalid BSP element size"),
        }
    }
}

impl Error for BspError {}
