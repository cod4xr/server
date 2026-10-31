// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::bsp::{BspBudget, BspError, BspLayout, BspLump, BspResource, LumpId};

const LEGACY_COUNTS: [usize; 13] = [41, 41, 42, 43, 43, 43, 43, 44, 44, 44, 46, 46, 47];

pub(super) fn parse(
    bytes: &[u8],
    budget: BspBudget,
) -> Result<(u32, BspLayout, Vec<BspLump<'_>>), BspError> {
    check(
        BspResource::FileBytes,
        bytes.len() as u64,
        budget.max_file_bytes,
    )?;
    if bytes.get(..4).ok_or(BspError::Truncated)? != b"IBSP" {
        return Err(BspError::InvalidMagic);
    }
    let version = word(bytes, 4)?;
    let (layout, count, start) = match version {
        6..=18 => {
            let index = usize::try_from(version - 6).map_err(|_| BspError::SizeOverflow)?;
            (BspLayout::OffsetDirectory, LEGACY_COUNTS[index], 8_usize)
        }
        19..=45 => (
            BspLayout::SequentialChunks,
            usize::try_from(word(bytes, 8)?).map_err(|_| BspError::SizeOverflow)?,
            12_usize,
        ),
        _ => return Err(BspError::UnsupportedVersion),
    };
    check(BspResource::Lumps, count as u64, budget.max_lumps as u64)?;
    let directory_end = count
        .checked_mul(8)
        .and_then(|size| start.checked_add(size))
        .ok_or(BspError::SizeOverflow)?;
    if directory_end > bytes.len() {
        return Err(BspError::Truncated);
    }
    let mut lumps = Vec::new();
    lumps
        .try_reserve_exact(count)
        .map_err(|_| BspError::AllocationFailed)?;
    let mut cursor = directory_end as u64;
    let mut total = 0_u64;
    for index in 0..count {
        let entry = start + index * 8;
        let first = word(bytes, entry)?;
        let second = word(bytes, entry + 4)?;
        let (id, offset, length) = match layout {
            BspLayout::OffsetDirectory => (
                u32::try_from(index).map_err(|_| BspError::SizeOverflow)?,
                u64::from(second),
                u64::from(first),
            ),
            BspLayout::SequentialChunks => (first, cursor, u64::from(second)),
        };
        check(BspResource::LumpBytes, length, budget.max_lump_bytes)?;
        total = total.checked_add(length).ok_or(BspError::SizeOverflow)?;
        check(
            BspResource::TotalLumpBytes,
            total,
            budget.max_total_lump_bytes,
        )?;
        let payload = if length == 0 {
            &bytes[..0]
        } else {
            if offset < directory_end as u64 {
                return Err(BspError::DirectoryOverlap);
            }
            let end = offset.checked_add(length).ok_or(BspError::SizeOverflow)?;
            let offset = usize::try_from(offset).map_err(|_| BspError::SizeOverflow)?;
            let end = usize::try_from(end).map_err(|_| BspError::SizeOverflow)?;
            bytes.get(offset..end).ok_or(BspError::Truncated)?
        };
        lumps.push(BspLump {
            id: LumpId::from_raw(id),
            offset,
            bytes: payload,
        });
        if layout == BspLayout::SequentialChunks {
            let aligned = length.checked_add(3).ok_or(BspError::SizeOverflow)? & !3;
            cursor = cursor.checked_add(aligned).ok_or(BspError::SizeOverflow)?;
        }
    }
    Ok((version, layout, lumps))
}

fn word(bytes: &[u8], offset: usize) -> Result<u32, BspError> {
    let end = offset.checked_add(4).ok_or(BspError::SizeOverflow)?;
    let value = bytes.get(offset..end).ok_or(BspError::Truncated)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn check(resource: BspResource, value: u64, limit: u64) -> Result<(), BspError> {
    if value > limit {
        return Err(BspError::BudgetExceeded(resource));
    }
    Ok(())
}
