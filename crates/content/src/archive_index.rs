// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;
use std::io::Cursor;

use zip::ZipArchive;

use crate::{ArchiveBudget, ArchiveError, VirtualPath};

pub(super) struct IndexedEntry {
    pub(super) index: usize,
    pub(super) offset: u64,
}

pub(super) type ArchiveIndex = HashMap<Vec<u8>, IndexedEntry>;

pub(super) fn build(
    archive: &ZipArchive<Cursor<&[u8]>>,
    budget: ArchiveBudget,
) -> Result<ArchiveIndex, ArchiveError> {
    let mut index = ArchiveIndex::new();
    index
        .try_reserve(archive.len())
        .map_err(|_| ArchiveError::AllocationFailed)?;
    for file_index in 0..archive.len() {
        let entry = archive
            .by_index_data(file_index)
            .map_err(|_| ArchiveError::InvalidArchive)?;
        if entry.is_dir() {
            continue;
        }
        let name = VirtualPath::try_new(entry.name_raw(), budget.max_path_bytes)?;
        let candidate = IndexedEntry {
            index: file_index,
            offset: entry.central_header_start(),
        };
        // The ZIP index collapses exact duplicates while retaining first-insert
        // index order. Directory offsets preserve the last matching entry even
        // when exact duplicates interleave with case/separator aliases.
        if let Some(previous) = index.get_mut(name.as_bytes()) {
            if candidate.offset > previous.offset {
                *previous = candidate;
            }
        } else {
            index.insert(name.into_bytes(), candidate);
        }
    }
    Ok(index)
}
