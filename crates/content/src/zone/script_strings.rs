// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::zone::prefix_cursor::PrefixCursor;
use crate::zone::zone_prefix_error;
use crate::zone::{ZonePrefixBudget, ZonePrefixError, ZonePrefixResource, ZoneString};

pub(super) fn read<'payload>(
    cursor: &mut PrefixCursor<'payload>,
    count: usize,
    budget: ZonePrefixBudget,
    block_limit: u64,
) -> Result<(Vec<ZoneString<'payload>>, u64), ZonePrefixError> {
    let table_bytes = count.checked_mul(4).ok_or(ZonePrefixError::SizeOverflow)?;
    let table = cursor.take(table_bytes)?;
    let metadata_bytes = count
        .checked_mul(size_of::<ZoneString<'_>>())
        .ok_or(ZonePrefixError::SizeOverflow)?;
    zone_prefix_error::check(
        ZonePrefixResource::MetadataBytes,
        metadata_bytes as u64,
        budget.max_metadata_bytes,
    )?;
    if table_bytes as u64 > block_limit {
        return Err(ZonePrefixError::BlockBounds);
    }
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(count)
        .map_err(|_| ZonePrefixError::AllocationFailed)?;
    let mut total_string_bytes = 0_u64;
    let mut block_bytes = table_bytes as u64;
    for bytes in table.chunks_exact(4) {
        let raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let slot = match raw {
            0 => ZoneString::Null,
            u32::MAX => {
                let string = cursor.string(budget.max_string_bytes)?;
                let size = string.len() as u64 + 1;
                total_string_bytes = total_string_bytes
                    .checked_add(size)
                    .ok_or(ZonePrefixError::SizeOverflow)?;
                zone_prefix_error::check(
                    ZonePrefixResource::TotalStringBytes,
                    total_string_bytes,
                    budget.max_total_string_bytes,
                )?;
                block_bytes = block_bytes
                    .checked_add(size)
                    .ok_or(ZonePrefixError::SizeOverflow)?;
                if block_bytes > block_limit {
                    return Err(ZonePrefixError::BlockBounds);
                }
                ZoneString::Inline(string)
            }
            encoded => ZoneString::Unresolved(encoded),
        };
        slots.push(slot);
    }
    Ok((slots, block_bytes))
}
