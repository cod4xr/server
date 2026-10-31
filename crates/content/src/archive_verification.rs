// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io::{Cursor, Read};

use zip::ZipReadOptions;
use zip::unstable::read::ZipIterable;

use crate::ArchiveError;
use crate::archive_preflight;

/// Streams every original entry, including hidden duplicates, through the ZIP
/// decoder. The source already passed admission and cannot change afterward.
pub(super) fn verify(bytes: &[u8]) -> Result<(), ArchiveError> {
    let mut archive = ZipIterable::try_new(Cursor::new(bytes), archive_preflight::config())
        .map_err(|_| ArchiveError::InvalidArchive)?;
    let entries = archive.files().map_err(|_| ArchiveError::InvalidArchive)?;
    while let Some(entry) = entries.next() {
        let entry = entry.map_err(|_| ArchiveError::InvalidArchive)?;
        let expected = entry.size();
        let mut file = entry
            .with_reader(entries.reader_mut(), ZipReadOptions::default())
            .map_err(|_| ArchiveError::InvalidArchive)?;
        verify_output(&mut file, expected)?;
    }
    Ok(())
}

fn verify_output(reader: &mut impl Read, expected: u64) -> Result<(), ArchiveError> {
    let mut scratch = [0; 16 * 1024];
    let mut total = 0_u64;
    loop {
        // Limit the final read to one detection byte, even if a forged stream
        // expands far beyond its declared size. No output is retained.
        let remaining = expected.saturating_sub(total).saturating_add(1);
        let limit = usize::try_from(remaining.min(scratch.len() as u64))
            .map_err(|_| ArchiveError::SizeOverflow)?;
        let read = reader
            .read(&mut scratch[..limit])
            .map_err(|_| ArchiveError::InvalidArchive)?;
        if read == 0 {
            return if total == expected {
                Ok(())
            } else {
                Err(ArchiveError::InvalidArchive)
            };
        }
        total = total
            .checked_add(read as u64)
            .ok_or(ArchiveError::SizeOverflow)?;
        if total > expected {
            return Err(ArchiveError::InvalidArchive);
        }
    }
}
