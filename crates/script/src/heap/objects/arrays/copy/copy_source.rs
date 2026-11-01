// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_lookup;
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind, StringHeap};

pub(super) struct CopySource {
    pub(super) first: Option<usize>,
    pub(super) count: usize,
    pub(super) bytes: u64,
}

pub(super) fn inspect(
    heap: &ObjectHeap,
    owner: usize,
    strings: &StringHeap,
    work: &mut ObjectWork,
) -> Result<CopySource, ObjectHeapErrorKind> {
    let Some(ObjectState::Occupied { first, fields }) =
        heap.objects.get(owner).map(|slot| &slot.state)
    else {
        return Err(ObjectHeapErrorKind::InvalidState);
    };
    let (mut next, mut previous, mut visited, mut bytes) = (*first, None, 0, 0_u64);
    while let Some(index) = next {
        work.charge(1)?;
        if visited >= *fields {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        let Some(FieldSlot::Occupied(field)) = heap.fields.get(index) else {
            return Err(ObjectHeapErrorKind::InvalidState);
        };
        if field.owner != owner || field.previous != previous {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        work.charge(field.key.len() as u64)?;
        FieldKey::decode_array(&field.key)?;
        object_lookup::validate_value(heap, field.value, strings, work)?;
        bytes = bytes
            .checked_add(field.key.len() as u64)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        visited += 1;
        previous = Some(index);
        next = field.next;
    }
    if visited != *fields {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    Ok(CopySource {
        first: *first,
        count: *fields,
        bytes,
    })
}
