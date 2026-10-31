// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::io::{Cursor, Read};

use crc32fast::Hasher;
use zip::ZipArchive;

use crate::archive_index::{self, ArchiveIndex};
use crate::archive_preflight;
use crate::archive_verification;
use crate::{ArchiveBudget, ArchiveError, ArchiveResource, ArchiveUsage, VirtualPath};

/// Validated read-only IWD backed by immutable caller-owned ZIP bytes.
///
/// The last directory entry matching a canonical path wins. Opening validates
/// every entry, including hidden duplicates. Decompression and CRC validation
/// occur on read. No files are extracted and no game formats are interpreted.
pub struct IwdArchive<'a> {
    bytes: &'a [u8],
    archive: ZipArchive<Cursor<&'a [u8]>>,
    index: ArchiveIndex,
    budget: ArchiveBudget,
    usage: ArchiveUsage,
    checksums: Vec<u32>,
}

impl<'a> IwdArchive<'a> {
    /// Validates metadata budgets/paths/headers before constructing the index.
    ///
    /// # Errors
    /// Rejects invalid/unsupported ZIP entries or exceeded budgets without
    /// publishing a partial archive. Library metadata allocations are bounded
    /// by admitted entry/directory sizes but are not all fallible allocations.
    pub fn open(bytes: &'a [u8], budget: ArchiveBudget) -> Result<Self, ArchiveError> {
        let (mut usage, checksums) = archive_preflight::validate(bytes, budget)?;
        let archive = ZipArchive::with_config(archive_preflight::config(), Cursor::new(bytes))
            .map_err(|_| ArchiveError::InvalidArchive)?;
        let index = archive_index::build(&archive, budget)?;
        usage.files = index.len();
        Ok(Self {
            bytes,
            archive,
            index,
            budget,
            usage,
            checksums,
        })
    }

    /// Returns archive admission statistics without payload access.
    #[must_use]
    pub fn usage(&self) -> ArchiveUsage {
        self.usage
    }

    /// Computes the legacy IWD checksum with the supplied pure-server feed.
    ///
    /// Hashes little-endian CRC32 values for every nonempty directory entry in
    /// original order, including overridden duplicates. Feed zero is the ordinary
    /// checksum. This is a compatibility checksum, not content authentication;
    /// it uses declared metadata and does not verify each entry's payload.
    #[must_use]
    pub fn legacy_checksum(&self, feed: u32) -> u32 {
        let mut hasher = Hasher::new_with_initial(feed);
        for checksum in &self.checksums {
            hasher.update(&checksum.to_le_bytes());
        }
        hasher.finalize()
    }

    /// Streams and validates every original payload, including hidden duplicates.
    ///
    /// Uses a 16 KiB scratch buffer plus decoder/one-entry metadata, without
    /// retaining decompressed bytes. Work is bounded by admitted entry/archive
    /// sizes and declared total output. Every call re-verifies the source.
    ///
    /// # Errors
    /// Returns corruption, CRC, truncation, or output-size errors without
    /// modifying the archive or returning any content. No authentication is implied.
    pub fn verify_all(&self) -> Result<(), ArchiveError> {
        archive_verification::verify(self.bytes)
    }

    /// Reports whether a normalized virtual file exists without allocation.
    #[must_use]
    pub fn contains(&self, path: &VirtualPath) -> bool {
        path.as_bytes().len() <= self.budget.max_path_bytes
            && self.index.contains_key(path.as_bytes())
    }

    /// Returns fully validated owned output, with no partial bytes on failure.
    ///
    /// Reads at most the declared size plus one detection byte. The allocation
    /// is fallible; configured single-entry output limits apply on every call.
    /// Successful output belongs to the caller and is not retained/cached here.
    ///
    /// # Errors
    /// Returns missing-path, allocation, budget, CRC, decompression, truncation,
    /// or size-mismatch errors. An error does not invalidate subsequent reads.
    pub fn read(&mut self, path: &VirtualPath) -> Result<Vec<u8>, ArchiveError> {
        if path.as_bytes().len() > self.budget.max_path_bytes {
            return Err(ArchiveError::BudgetExceeded {
                resource: ArchiveResource::PathBytes,
            });
        }
        let entry = self
            .index
            .get(path.as_bytes())
            .ok_or(ArchiveError::NotFound)?;
        let mut file = self
            .archive
            .by_index(entry.index)
            .map_err(|_| ArchiveError::InvalidArchive)?;
        let size = usize::try_from(file.size()).map_err(|_| ArchiveError::SizeOverflow)?;
        if file.size() > self.budget.max_entry_bytes {
            return Err(ArchiveError::BudgetExceeded {
                resource: ArchiveResource::EntryBytes,
            });
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(size)
            .map_err(|_| ArchiveError::AllocationFailed)?;
        output.resize(size, 0);
        file.read_exact(&mut output)
            .map_err(|_| ArchiveError::InvalidArchive)?;
        // A nonempty read forces CRC/length validation, including empty files.
        let mut detection = [0];
        if file
            .read(&mut detection)
            .map_err(|_| ArchiveError::InvalidArchive)?
            != 0
        {
            return Err(ArchiveError::InvalidArchive);
        }
        Ok(output)
    }
}

impl fmt::Debug for IwdArchive<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IwdArchive")
            .field("usage", &self.usage)
            .field("budget", &self.budget)
            .finish_non_exhaustive()
    }
}
