// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::objects::index_rebuild;
use crate::heap::objects::object_slot::{Field, FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{ObjectHeap, ObjectHeapErrorKind};

pub(super) fn insert(
    heap: &mut ObjectHeap,
    owner: usize,
    hash: u64,
    key: &[u8],
    value: KernelValue,
    bucket: Option<usize>,
    work: &mut ObjectWork,
) -> Result<(), ObjectHeapErrorKind> {
    let (first, count) = match heap.objects.get(owner).map(|slot| &slot.state) {
        Some(ObjectState::Occupied { first, fields }) => (*first, *fields),
        _ => return Err(ObjectHeapErrorKind::InvalidState),
    };
    if let Some(first) = first
        && !matches!(heap.fields.get(first), Some(FieldSlot::Occupied(field)) if field.owner == owner && field.previous.is_none())
    {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    let count = count
        .checked_add(1)
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    let slot = heap.free_field.unwrap_or(heap.fields.len());
    let next_free = match heap.free_field {
        Some(index) => match heap.fields.get(index) {
            Some(FieldSlot::Vacant { next }) => *next,
            _ => return Err(ObjectHeapErrorKind::InvalidState),
        },
        None => None,
    };
    let length = heap.index.target_length()?;
    let mut usage = heap.usage;
    usage.live_fields = usage
        .live_fields
        .checked_add(1)
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    usage.key_bytes = usage
        .key_bytes
        .checked_add(key.len() as u64)
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    if heap.free_field.is_none() {
        usage.field_slots = usage
            .field_slots
            .checked_add(1)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    }
    usage.index_buckets = length as u64;
    let usage = usage.admit(heap.budget)?;
    let mut staged = if heap.index.needs_rebuild() {
        Some(index_rebuild::stage(
            length,
            &heap.fields,
            &heap.objects,
            work,
        )?)
    } else {
        None
    };
    let position = if let Some(index) = &staged {
        index.empty_position(hash, work)?
    } else {
        bucket.ok_or(ObjectHeapErrorKind::InvalidState)?
    };
    let bytes = copy_key(key, work)?;
    if heap.free_field.is_none() {
        if heap.fields.len() == heap.fields.capacity() {
            work.charge(heap.fields.len() as u64)?;
        }
        heap.fields
            .try_reserve(1)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
    }
    work.charge(1)?;
    if let Some(index) = &mut staged {
        index.insert(position, slot)?;
    } else {
        heap.index.insert(position, slot)?;
    }
    // All references, links, counts, index positions and capacity were validated
    // above. Publication below cannot allocate or consume more work.
    let field = FieldSlot::Occupied(Field {
        owner,
        hash,
        key: bytes,
        value,
        previous: None,
        next: first,
    });
    if heap.free_field.is_none() {
        heap.fields.push(field);
    } else {
        heap.fields[slot] = field;
    }
    if let Some(first) = first
        && let FieldSlot::Occupied(field) = &mut heap.fields[first]
    {
        field.previous = Some(slot);
    }
    heap.objects[owner].state = ObjectState::Occupied {
        first: Some(slot),
        fields: count,
    };
    heap.free_field = next_free;
    if let Some(staged) = staged {
        heap.index = staged;
    }
    heap.usage = usage;
    Ok(())
}

fn copy_key(key: &[u8], work: &mut ObjectWork) -> Result<Vec<u8>, ObjectHeapErrorKind> {
    work.charge(key.len() as u64)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(key.len())
        .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
    bytes.extend_from_slice(key);
    Ok(bytes)
}
