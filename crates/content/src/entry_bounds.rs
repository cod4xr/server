// SPDX-License-Identifier: AGPL-3.0-or-later
use zip::read::ZipFileEntry;

use crate::ArchiveError;

/// Local headers already passed library consistency checks. Independently bound
/// their compressed payload spans to the immutable pre-directory archive region.
pub(super) fn validate(
    bytes: &[u8],
    entry: &ZipFileEntry<'_>,
    directory_start: u64,
) -> Result<(), ArchiveError> {
    let start = usize::try_from(entry.header_start()).map_err(|_| ArchiveError::SizeOverflow)?;
    let fixed_end = start.checked_add(30).ok_or(ArchiveError::SizeOverflow)?;
    let fixed = bytes
        .get(start..fixed_end)
        .ok_or(ArchiveError::InvalidArchive)?;
    let name_bytes = u16::from_le_bytes([fixed[26], fixed[27]]);
    let extra_bytes = u16::from_le_bytes([fixed[28], fixed[29]]);
    let end = (fixed_end as u64)
        .checked_add(u64::from(name_bytes))
        .and_then(|end| end.checked_add(u64::from(extra_bytes)))
        .and_then(|end| end.checked_add(entry.compressed_size()))
        .ok_or(ArchiveError::SizeOverflow)?;
    if end > directory_start {
        return Err(ArchiveError::InvalidArchive);
    }
    Ok(())
}
