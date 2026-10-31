// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Resource rejected by the configured archive policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArchiveResource {
    /// Caller-owned encoded archive length.
    ArchiveBytes,
    /// Total declared compressed work, including overlapping/repeated entries.
    CompressedBytes,
    /// Central directory and footer bytes.
    MetadataBytes,
    /// Directory entry count, including duplicates.
    Entries,
    /// A single canonical virtual path.
    PathBytes,
    /// Total raw filename bytes.
    NameBytes,
    /// One entry's declared output length.
    EntryBytes,
    /// Sum of declared output lengths.
    TotalBytes,
}

/// Sanitized archive failure, never including remote names or payload bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArchiveError {
    /// Malformed, truncated, inconsistent, or corrupt ZIP data.
    InvalidArchive,
    /// Absolute, empty, traversal, control-byte, or ambiguous virtual path.
    InvalidPath,
    /// Encrypted, symlink, or unsupported compression entry.
    UnsupportedEntry,
    /// Configured resource admission failed before publishing an archive.
    BudgetExceeded {
        /// Resource whose configured limit was exceeded.
        resource: ArchiveResource,
    },
    /// Fallible index or output allocation failed.
    AllocationFailed,
    /// Host representation cannot hold a declared size or arithmetic overflowed.
    SizeOverflow,
    /// No file has this canonical path.
    NotFound,
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArchive => formatter.write_str("invalid or corrupt content archive"),
            Self::InvalidPath => formatter.write_str("invalid content path"),
            Self::UnsupportedEntry => formatter.write_str("unsupported content entry"),
            Self::BudgetExceeded { resource } => {
                write!(formatter, "content budget exceeded: {resource:?}")
            }
            Self::AllocationFailed => formatter.write_str("content allocation failed"),
            Self::SizeOverflow => formatter.write_str("content size cannot be represented"),
            Self::NotFound => formatter.write_str("content entry not found"),
        }
    }
}

impl Error for ArchiveError {}
