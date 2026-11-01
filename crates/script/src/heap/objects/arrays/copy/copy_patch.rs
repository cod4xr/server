// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::copy::copy_index::{CopyIndex, CopyIndexLayout};
use crate::heap::objects::arrays::copy::copy_occupancy::CopyOccupancy;
use crate::heap::objects::arrays::copy::copy_source::CopySource;
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_slot::FieldSlot;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind};

/// Simulate actual empty/deleted placements before choosing a global rebuild.
/// Resetting the small occupancy set reuses its fallible allocation for staging.
pub(super) fn stage(
    heap: &ObjectHeap,
    layout: CopyIndexLayout,
    source: &CopySource,
    owner: usize,
    records: u64,
    work: &mut ObjectWork,
) -> Result<Option<CopyIndex>, ObjectHeapErrorKind> {
    let capacity = CopyOccupancy::capacity(layout.count)?;
    let scratch = (capacity as u64)
        .checked_mul(size_of::<Option<usize>>() as u64)
        .and_then(|bytes| records.checked_add(bytes))
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    work.scratch(scratch)?;
    let mut staged = CopyIndex::Patched {
        occupied: CopyOccupancy::new(capacity, work)?,
        live: layout.live,
        filled: heap.index.filled,
    };
    let (mut next, mut visited) = (source.first, 0);
    while let Some(index) = next {
        work.charge(1)?;
        if visited >= source.count {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        let Some(FieldSlot::Occupied(field)) = heap.fields.get(index) else {
            return Err(ObjectHeapErrorKind::InvalidState);
        };
        work.charge(field.key.len() as u64)?;
        let hash = FieldKey::Array(FieldKey::decode_array(&field.key)?).hash(owner);
        staged.place(heap, hash, 0, work)?;
        visited += 1;
        next = field.next;
    }
    if visited != source.count {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    let CopyIndex::Patched {
        occupied, filled, ..
    } = &mut staged
    else {
        return Err(ObjectHeapErrorKind::InvalidState);
    };
    if *filled > layout.length / 2 {
        return Ok(None);
    }
    occupied.clear(work)?;
    *filled = heap.index.filled;
    Ok(Some(staged))
}
