// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::fastfile::FastFileStream;
use crate::zone::rawfiles::raw_file_body::RawFileBodyReader;
use crate::zone::rawfiles::{RawFile, RawFileBudget, RawFileError, RawFileUsage};
use crate::zone::{ZonePointer, ZonePrefix};

/// Complete borrowed raw-file-only root directory, without engine registration.
///
/// Null entries and aliases retain directory order. Repeated views share source
/// bytes; this does not implement duplicate-name registry precedence or interning.
pub struct RawFileCollection<'payload> {
    entries: Vec<Option<RawFile<'payload>>>,
    usage: RawFileUsage,
}

impl<'payload> RawFileCollection<'payload> {
    /// Decodes the supported collection completely before returning any raw file.
    ///
    /// Only raw-file roots (type 31) are supported. Direct names resolve through
    /// known inline string regions, while asset aliases resolve recorded pointer
    /// slots. Null slots may be referenced before their directory turn; other
    /// targets must already be loaded. No native pointers or engine IDs are made.
    ///
    /// # Errors
    /// Rejects unsupported kinds/reference cases, malformed lengths/terminators,
    /// truncated/excess data, block/budget overflow, and fallible allocation failure.
    /// No partial collection is returned; the source remains unchanged. This does
    /// not load mixed stock zones, validate script syntax, or execute gameplay.
    pub fn decode(
        stream: &'payload FastFileStream,
        budget: RawFileBudget,
    ) -> Result<Self, RawFileError> {
        let prefix = ZonePrefix::parse(stream, budget.prefix).map_err(RawFileError::Prefix)?;
        if prefix.assets().any(|record| record.kind() != 31) {
            return Err(RawFileError::UnsupportedAsset);
        }
        let count = prefix.assets().len();
        let directory_bytes = (count as u64)
            .checked_mul(8)
            .ok_or(RawFileError::SizeOverflow)?;
        let directory_start = prefix
            .virtual_bytes()
            .checked_sub(directory_bytes)
            .ok_or(RawFileError::SizeOverflow)?;
        let mut reader = RawFileBodyReader::new(&prefix, stream.header(), budget)?;
        // The complete serialized table is already loaded. Encoded null cells
        // are safe forward aliases; unresolved non-null words are not pointees.
        for (index, record) in prefix.assets().enumerate() {
            if record.encoded_header() == 0 {
                reader.bind(header_offset(directory_start, index)?, None)?;
            }
        }
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| RawFileError::AllocationFailed)?;
        for (index, record) in prefix.assets().enumerate() {
            let file = match record.decode_header()? {
                ZonePointer::Null => None,
                ZonePointer::Offset(address) => reader.alias(address)?,
                ZonePointer::Inline => Some(reader.read(false)?),
                ZonePointer::InlineWithAlias => Some(reader.read(true)?),
            };
            reader.bind(header_offset(directory_start, index)?, file)?;
            entries.push(file);
        }
        if !reader.is_finished() {
            return Err(RawFileError::TrailingData);
        }
        Ok(Self {
            usage: reader.usage(count),
            entries,
        })
    }

    /// Borrows all root slots, preserving null entries and alias order.
    #[must_use]
    pub fn entries(&self) -> &[Option<RawFile<'payload>>] {
        &self.entries
    }

    /// Returns logical decode/work accounting; staging metadata is no longer live.
    #[must_use]
    pub fn usage(&self) -> RawFileUsage {
        self.usage
    }
}

fn header_offset(start: u64, index: usize) -> Result<u64, RawFileError> {
    (index as u64)
        .checked_mul(8)
        .and_then(|offset| offset.checked_add(4))
        .and_then(|offset| start.checked_add(offset))
        .ok_or(RawFileError::SizeOverflow)
}

impl fmt::Debug for RawFileCollection<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawFileCollection")
            .field("root_entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}
