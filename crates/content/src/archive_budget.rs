// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ArchiveError, ArchiveResource};

/// Configurable admission limits, independent of legacy asset/wire capacities.
///
/// Counters bound encoded bytes, declared output, and index work, not process
/// RSS. Zero admits none of the corresponding resource; no data is truncated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveBudget {
    /// Encoded archive length; storage remains borrowed from the caller.
    pub max_archive_bytes: u64,
    /// Sum of declared compressed payload lengths, including duplicates.
    pub max_compressed_bytes: u64,
    /// Central-directory bytes through the archive footer, including comments.
    pub max_metadata_bytes: u64,
    /// All directory entries, including directories and overridden duplicates.
    pub max_entries: usize,
    /// Bytes per virtual name, without a directory's final separator.
    pub max_path_bytes: usize,
    /// Sum of original filename lengths, including duplicates/directories.
    pub max_name_bytes: u64,
    /// Declared output bytes of a single entry.
    pub max_entry_bytes: u64,
    /// Sum of declared output bytes, including overridden entries.
    pub max_total_bytes: u64,
}

impl Default for ArchiveBudget {
    fn default() -> Self {
        Self {
            max_archive_bytes: 2 * 1024 * 1024 * 1024,
            max_compressed_bytes: 2 * 1024 * 1024 * 1024,
            max_metadata_bytes: 64 * 1024 * 1024,
            max_entries: 100_000,
            max_path_bytes: 4096,
            max_name_bytes: 32 * 1024 * 1024,
            max_entry_bytes: 256 * 1024 * 1024,
            max_total_bytes: 8 * 1024 * 1024 * 1024,
        }
    }
}

pub(super) fn check(
    resource: ArchiveResource,
    requested: u64,
    limit: u64,
) -> Result<(), ArchiveError> {
    if requested > limit {
        return Err(ArchiveError::BudgetExceeded { resource });
    }
    Ok(())
}
