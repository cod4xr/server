// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::ArrayValueState;
use crate::heap::objects::object_slot::ObjectState;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind};

pub(in crate::heap::objects) fn preflight(
    heap: &ObjectHeap,
    flags: &[bool],
    work: &mut ObjectWork,
) -> Result<u64, ObjectHeapErrorKind> {
    let arrays = &heap.arrays;
    if arrays.metadata.len() as u64 != heap.usage.array_metadata_rows
        || arrays.values.len() as u64 != heap.usage.array_value_slots
    {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    if arrays.metadata.is_empty() {
        if !arrays.values.is_empty()
            || heap.usage.live_arrays != 0
            || heap.usage.live_array_values != 0
        {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        return Ok(0);
    }
    if arrays.metadata.len() != heap.objects.len() || arrays.owner.is_none() {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    let (mut total, mut live, mut owners) = (0_u64, 0_u64, 0_u64);
    for (index, metadata) in arrays.metadata.iter().enumerate() {
        work.charge(1)?;
        if metadata.is_array {
            if !matches!(heap.objects[index].state, ObjectState::Occupied { .. }) {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            total = total
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            owners = owners
                .checked_add(metadata.owners)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            if flags[index] {
                live = live
                    .checked_add(1)
                    .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            } else if metadata.owners != 0 {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
        } else if metadata.owners != 0 {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
    }
    let mut values = 0_u64;
    for slot in &arrays.values {
        work.charge(1)?;
        if let ArrayValueState::Occupied(id) = slot.state {
            work.charge(1)?;
            let index = heap.resolve(id)?;
            if !arrays.is_array(index) || arrays.metadata[index].owners == 0 || !flags[index] {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            values = values
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        }
    }
    if total != heap.usage.live_arrays || values != heap.usage.live_array_values || owners != values
    {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    Ok(live)
}
