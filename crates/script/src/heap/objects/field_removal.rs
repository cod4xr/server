// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::objects::field_index::{Bucket, FieldIndex};
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{ObjectHeap, ObjectHeapErrorKind};

pub(super) fn remove(
    heap: &mut ObjectHeap,
    owner: usize,
    index: usize,
    bucket: usize,
    work: &mut ObjectWork,
) -> Result<KernelValue, ObjectHeapErrorKind> {
    work.charge(1)?;
    let Some(FieldSlot::Occupied(field)) = heap.fields.get(index) else {
        return Err(ObjectHeapErrorKind::InvalidState);
    };
    if field.owner != owner
        || !matches!(heap.index.buckets.get(bucket), Some(Bucket::Occupied(found)) if *found == index)
    {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    let (previous, next, bytes, value) = (field.previous, field.next, field.key.len(), field.value);
    let (first, count) = match heap.objects.get(owner).map(|slot| &slot.state) {
        Some(ObjectState::Occupied { first, fields }) => (
            *first,
            fields
                .checked_sub(1)
                .ok_or(ObjectHeapErrorKind::InvalidState)?,
        ),
        _ => return Err(ObjectHeapErrorKind::InvalidState),
    };
    if let Some(previous) = previous {
        if !matches!(heap.fields.get(previous), Some(FieldSlot::Occupied(field)) if field.owner == owner && field.next == Some(index))
        {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
    } else if first != Some(index) {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    if let Some(next) = next
        && !matches!(heap.fields.get(next), Some(FieldSlot::Occupied(field)) if field.owner == owner && field.previous == Some(index))
    {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    let live = heap
        .index
        .live
        .checked_sub(1)
        .ok_or(ObjectHeapErrorKind::InvalidState)?;
    let mut usage = heap.usage;
    usage.live_fields = usage
        .live_fields
        .checked_sub(1)
        .ok_or(ObjectHeapErrorKind::InvalidState)?;
    usage.key_bytes = usage
        .key_bytes
        .checked_sub(bytes as u64)
        .ok_or(ObjectHeapErrorKind::InvalidState)?;
    if live == 0 {
        usage.index_buckets = 0;
    }
    let usage = usage.admit(heap.budget)?;
    // All links/positions and decreasing counters are checked before mutation.
    heap.index.buckets[bucket] = Bucket::Deleted;
    heap.index.live = live;
    if let Some(previous) = previous
        && let FieldSlot::Occupied(field) = &mut heap.fields[previous]
    {
        field.next = next;
    }
    if let Some(next) = next
        && let FieldSlot::Occupied(field) = &mut heap.fields[next]
    {
        field.previous = previous;
    }
    heap.objects[owner].state = ObjectState::Occupied {
        first: if previous.is_none() { next } else { first },
        fields: count,
    };
    heap.fields[index] = FieldSlot::Vacant {
        next: heap.free_field,
    };
    heap.free_field = Some(index);
    if live == 0 {
        heap.index = FieldIndex::default();
    }
    heap.usage = usage;
    Ok(value)
}
