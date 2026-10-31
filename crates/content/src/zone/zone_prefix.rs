// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::fastfile::FastFileStream;
use crate::zone::prefix_cursor::PrefixCursor;
use crate::zone::script_strings;
use crate::zone::zone_prefix_error;
use crate::zone::{
    ZoneAssetRecord, ZonePrefixBudget, ZonePrefixError, ZonePrefixResource, ZoneString,
};

/// Borrowed script strings and root asset-directory metadata for one IW3 stream.
///
/// This owns only a fallibly allocated slot index. It does not intern strings,
/// resolve references, validate asset types/bodies, or register a zone for gameplay.
pub struct ZonePrefix<'payload> {
    strings: Vec<ZoneString<'payload>>,
    assets: &'payload [u8],
    remaining: &'payload [u8],
    virtual_bytes: u64,
}

impl<'payload> ZonePrefix<'payload> {
    /// Validates the root tables/inline string spans and declared virtual-block fit.
    ///
    /// Root table pointer words act as presence flags, matching the source loader;
    /// individual unresolved pointers and unknown asset kinds remain uninterpreted.
    /// Stream-block alignment advances logical allocation, not serialized input.
    ///
    /// # Errors
    /// Rejects negative counts, missing nonempty tables, budget/size/allocation
    /// failures, truncated metadata, and virtual-block overflow. Returns no partial
    /// prefix and does not mutate the borrowed stream on failure.
    pub fn parse(
        stream: &'payload FastFileStream,
        budget: ZonePrefixBudget,
    ) -> Result<Self, ZonePrefixError> {
        let mut cursor = PrefixCursor::new(stream.payload());
        let string_count = count(
            cursor.word()?,
            budget.max_script_strings,
            ZonePrefixResource::ScriptStrings,
        )?;
        let strings_present = cursor.word()? != 0;
        let asset_count = count(
            cursor.word()?,
            budget.max_assets,
            ZonePrefixResource::Assets,
        )?;
        let assets_present = cursor.word()? != 0;
        if (string_count > 0 && !strings_present) || (asset_count > 0 && !assets_present) {
            return Err(ZonePrefixError::MissingTable);
        }
        let block_limit = stream.header().block_bytes()[4];
        let (strings, mut virtual_bytes) =
            script_strings::read(&mut cursor, string_count, budget, block_limit)?;
        if assets_present {
            virtual_bytes = virtual_bytes
                .checked_add(3)
                .ok_or(ZonePrefixError::SizeOverflow)?
                & !3;
        }
        let asset_bytes = asset_count
            .checked_mul(8)
            .ok_or(ZonePrefixError::SizeOverflow)?;
        virtual_bytes = virtual_bytes
            .checked_add(asset_bytes as u64)
            .ok_or(ZonePrefixError::SizeOverflow)?;
        if virtual_bytes > block_limit {
            return Err(ZonePrefixError::BlockBounds);
        }
        let assets = cursor.take(asset_bytes)?;
        Ok(Self {
            strings,
            assets,
            remaining: cursor.remaining(),
            virtual_bytes,
        })
    }

    /// Returns every serialized script slot without narrowing to legacy u16 handles.
    #[must_use]
    pub fn strings(&self) -> &[ZoneString<'payload>] {
        &self.strings
    }

    /// Iterates every root record in order, including unknown kinds and null headers.
    pub fn assets(
        &self,
    ) -> impl ExactSizeIterator<Item = ZoneAssetRecord> + DoubleEndedIterator + '_ {
        self.assets.chunks_exact(8).map(ZoneAssetRecord::parse)
    }

    /// Borrows the unvalidated body/deferred-data remainder without copying it.
    #[must_use]
    pub fn remaining(&self) -> &'payload [u8] {
        self.remaining
    }

    /// Returns virtual-block bytes consumed by this prefix, including alignment gaps.
    #[must_use]
    pub fn virtual_bytes(&self) -> u64 {
        self.virtual_bytes
    }
}

fn count(raw: u32, maximum: u64, resource: ZonePrefixResource) -> Result<usize, ZonePrefixError> {
    if raw > i32::MAX as u32 {
        return Err(ZonePrefixError::InvalidCount);
    }
    zone_prefix_error::check(resource, u64::from(raw), maximum)?;
    usize::try_from(raw).map_err(|_| ZonePrefixError::SizeOverflow)
}

impl fmt::Debug for ZonePrefix<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZonePrefix")
            .field("script_strings", &self.strings.len())
            .field("asset_records", &(self.assets.len() / 8))
            .field("remaining_bytes", &self.remaining.len())
            .field("virtual_bytes", &self.virtual_bytes)
            .finish_non_exhaustive()
    }
}
