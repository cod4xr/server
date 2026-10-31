// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io::{Cursor, Seek};

use zip::CompressionMethod;
use zip::read::{ArchiveOffset, Config};
use zip::unstable::read::ZipIterable;

use crate::archive_budget;
use crate::entry_bounds;
use crate::{ArchiveBudget, ArchiveError, ArchiveResource, ArchiveUsage, VirtualPath};

pub(super) fn config() -> Config {
    Config {
        archive_offset: ArchiveOffset::Known(0),
    }
}

/// Completes a streaming metadata validation pass before the library allocates
/// its archive-wide index. Both passes use the same immutable bytes/configuration.
pub(super) fn validate(
    bytes: &[u8],
    budget: ArchiveBudget,
) -> Result<(ArchiveUsage, Vec<u32>), ArchiveError> {
    let mut usage = ArchiveUsage {
        archive_bytes: bytes.len() as u64,
        ..ArchiveUsage::default()
    };
    archive_budget::check(
        ArchiveResource::ArchiveBytes,
        usage.archive_bytes,
        budget.max_archive_bytes,
    )?;
    let mut archive = ZipIterable::try_new(Cursor::new(bytes), config())
        .map_err(|_| ArchiveError::InvalidArchive)?
        .enforce_header_consistency(true);
    usage.entries = archive.len();
    archive_budget::check(
        ArchiveResource::Entries,
        usage.entries as u64,
        budget.max_entries as u64,
    )?;
    let entries = archive.files().map_err(|_| ArchiveError::InvalidArchive)?;
    let directory_start = entries
        .reader_mut()
        .stream_position()
        .map_err(|_| ArchiveError::InvalidArchive)?;
    usage.metadata_bytes = usage
        .archive_bytes
        .checked_sub(directory_start)
        .ok_or(ArchiveError::InvalidArchive)?;
    archive_budget::check(
        ArchiveResource::MetadataBytes,
        usage.metadata_bytes,
        budget.max_metadata_bytes,
    )?;
    let mut checksums = Vec::new();
    checksums
        .try_reserve_exact(usage.entries)
        .map_err(|_| ArchiveError::AllocationFailed)?;
    for entry in entries {
        let entry = entry.map_err(|_| ArchiveError::InvalidArchive)?;
        entry_bounds::validate(bytes, &entry, directory_start)?;
        if entry.encrypted()
            || entry.is_symlink()
            || !matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err(ArchiveError::UnsupportedEntry);
        }
        let name = entry.name_raw();
        let path = if entry.is_dir() {
            if entry.size() != 0 {
                return Err(ArchiveError::UnsupportedEntry);
            }
            name.get(..name.len().saturating_sub(1))
                .ok_or(ArchiveError::InvalidPath)?
        } else {
            name
        };
        let _path = VirtualPath::try_new(path, budget.max_path_bytes)?;
        usage.name_bytes = usage
            .name_bytes
            .checked_add(name.len() as u64)
            .ok_or(ArchiveError::SizeOverflow)?;
        usage.declared_bytes = usage
            .declared_bytes
            .checked_add(entry.size())
            .ok_or(ArchiveError::SizeOverflow)?;
        usage.compressed_bytes = usage
            .compressed_bytes
            .checked_add(entry.compressed_size())
            .ok_or(ArchiveError::SizeOverflow)?;
        archive_budget::check(
            ArchiveResource::CompressedBytes,
            usage.compressed_bytes,
            budget.max_compressed_bytes,
        )?;
        archive_budget::check(
            ArchiveResource::NameBytes,
            usage.name_bytes,
            budget.max_name_bytes,
        )?;
        archive_budget::check(
            ArchiveResource::EntryBytes,
            entry.size(),
            budget.max_entry_bytes,
        )?;
        archive_budget::check(
            ArchiveResource::TotalBytes,
            usage.declared_bytes,
            budget.max_total_bytes,
        )?;
        if entry.size() != 0 {
            checksums.push(entry.crc32());
        }
    }
    Ok((usage, checksums))
}
