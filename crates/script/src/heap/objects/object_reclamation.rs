// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::HeapWorkBudget;
use crate::heap::objects::arrays::array_collection;
use crate::heap::objects::field_index::{Bucket, FieldIndex};
use crate::heap::objects::object_mark;
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{
    ObjectCollection, ObjectHeap, ObjectHeapError, ObjectHeapErrorKind, ObjectHeapUsage, ObjectId,
};

pub(super) fn reclaim(
    heap: &mut ObjectHeap,
    roots: &[ObjectId],
    budget: HeapWorkBudget,
) -> Result<ObjectCollection, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    collect(heap, roots, &mut work).map_err(|kind| work.error(kind))
}

fn collect(
    heap: &mut ObjectHeap,
    roots: &[ObjectId],
    work: &mut ObjectWork,
) -> Result<ObjectCollection, ObjectHeapErrorKind> {
    let flags = object_mark::trace(heap, heap.arrays.roots(roots), work)?;
    let next = preflight(heap, &flags, work)?;
    let sweep = heap
        .objects
        .len()
        .checked_add(heap.fields.len())
        .and_then(|slots| slots.checked_add(heap.index.buckets.len()))
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    work.charge(sweep as u64)?;
    let objects = heap
        .usage
        .live_objects
        .checked_sub(next.live_objects)
        .ok_or(ObjectHeapErrorKind::InvalidState)?;
    let fields = heap
        .usage
        .live_fields
        .checked_sub(next.live_fields)
        .ok_or(ObjectHeapErrorKind::InvalidState)?;
    let key_bytes = heap
        .usage
        .key_bytes
        .checked_sub(next.key_bytes)
        .ok_or(ObjectHeapErrorKind::InvalidState)?;
    let live_fields =
        usize::try_from(next.live_fields).map_err(|_| ObjectHeapErrorKind::SizeOverflow)?;
    // All references, links, counter changes and sweep work preflight above.
    // Flat field values/slots have no user callbacks or recursively owned objects.
    apply(heap, &flags, next, live_fields);
    Ok(ObjectCollection {
        objects,
        fields,
        key_bytes,
        usage: work.usage(),
    })
}

fn preflight(
    heap: &ObjectHeap,
    flags: &[bool],
    work: &mut ObjectWork,
) -> Result<ObjectHeapUsage, ObjectHeapErrorKind> {
    if flags.len() != heap.objects.len() || heap.index.live as u64 != heap.usage.live_fields {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    let mut next = heap.usage;
    next.live_arrays = array_collection::preflight(heap, flags, work)?;
    let (mut objects, mut fields, mut bytes) = (0_u64, 0_u64, 0_u64);
    next.live_objects = 0;
    next.live_fields = 0;
    next.key_bytes = 0;
    for (index, slot) in heap.objects.iter().enumerate() {
        work.charge(1)?;
        if matches!(slot.state, ObjectState::Occupied { .. }) {
            objects = objects
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            if flags[index] {
                next.live_objects = next
                    .live_objects
                    .checked_add(1)
                    .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            }
        } else if flags[index] {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
    }
    for slot in &heap.fields {
        work.charge(1)?;
        if let FieldSlot::Occupied(field) = slot {
            if heap.arrays.is_array(field.owner) {
                work.charge(1)?;
                crate::heap::objects::field_key::FieldKey::decode_array(&field.key)?;
            }
            if !matches!(
                heap.objects.get(field.owner).map(|slot| &slot.state),
                Some(ObjectState::Occupied { .. })
            ) {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            fields = fields
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            bytes = bytes
                .checked_add(field.key.len() as u64)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            if flags[field.owner] {
                next.live_fields = next
                    .live_fields
                    .checked_add(1)
                    .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
                next.key_bytes = next
                    .key_bytes
                    .checked_add(field.key.len() as u64)
                    .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            }
        }
    }
    let mut occupied = 0_u64;
    for bucket in &heap.index.buckets {
        work.charge(1)?;
        if let Bucket::Occupied(index) = bucket {
            if !matches!(heap.fields.get(*index), Some(FieldSlot::Occupied(_))) {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            occupied = occupied
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        }
    }
    if objects != heap.usage.live_objects
        || fields != heap.usage.live_fields
        || bytes != heap.usage.key_bytes
        || occupied != fields
    {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    if next.live_fields == 0 {
        next.index_buckets = 0;
    }
    next.admit(heap.budget)
}

fn apply(heap: &mut ObjectHeap, flags: &[bool], next: ObjectHeapUsage, live_fields: usize) {
    for bucket in &mut heap.index.buckets {
        if let Bucket::Occupied(index) = bucket
            && let FieldSlot::Occupied(field) = &heap.fields[*index]
            && !flags[field.owner]
        {
            *bucket = Bucket::Deleted;
        }
    }
    for (index, field) in heap.fields.iter_mut().enumerate() {
        if matches!(field, FieldSlot::Occupied(field) if !flags[field.owner]) {
            *field = FieldSlot::Vacant {
                next: heap.free_field,
            };
            heap.free_field = Some(index);
        }
    }
    for (index, slot) in heap.objects.iter_mut().enumerate() {
        if matches!(slot.state, ObjectState::Occupied { .. }) && !flags[index] {
            if let Some(metadata) = heap.arrays.metadata.get_mut(index) {
                *metadata = crate::heap::objects::arrays::ArrayMetadata::default();
            }
            if let Some(generation) = slot.generation.checked_add(1) {
                slot.generation = generation;
                slot.state = ObjectState::Vacant {
                    next: heap.free_object,
                };
                heap.free_object = Some(index);
            } else {
                slot.state = ObjectState::Retired;
            }
        }
    }
    if next.live_fields == 0 {
        heap.index = FieldIndex::default();
    } else {
        heap.index.live = live_fields;
    }
    heap.usage = next;
}
