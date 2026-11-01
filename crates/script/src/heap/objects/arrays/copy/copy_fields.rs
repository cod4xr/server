// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::copy::copy_index::CopyIndex;
use crate::heap::objects::arrays::copy::copy_source::CopySource;
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_slot::{Field, FieldSlot};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind};

pub(super) struct CopyField {
    pub(super) slot: usize,
    pub(super) bucket: usize,
    pub(super) field: Field,
}

pub(super) fn stage(
    heap: &ObjectHeap,
    owner: usize,
    source: &CopySource,
    index: &mut CopyIndex,
    work: &mut ObjectWork,
) -> Result<Vec<CopyField>, ObjectHeapErrorKind> {
    let mut records: Vec<CopyField> = Vec::new();
    records
        .try_reserve_exact(source.count)
        .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
    let (mut next, mut free, mut append) = (source.first, heap.free_field, heap.fields.len());
    while let Some(source_index) = next {
        work.charge(1)?;
        if records.len() >= source.count {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        let Some(FieldSlot::Occupied(field)) = heap.fields.get(source_index) else {
            return Err(ObjectHeapErrorKind::InvalidState);
        };
        let slot = if let Some(slot) = free {
            let Some(FieldSlot::Vacant { next }) = heap.fields.get(slot) else {
                return Err(ObjectHeapErrorKind::InvalidState);
            };
            free = *next;
            slot
        } else {
            let slot = append;
            append = append
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            slot
        };
        work.charge(field.key.len() as u64)?;
        let hash = FieldKey::Array(FieldKey::decode_array(&field.key)?).hash(owner);
        let bytes = FieldKey::Bytes(&field.key).copy(work)?;
        let bucket = index.place(heap, hash, slot, work)?;
        let first = records.last().map(|record| record.slot);
        if let Some(previous) = records.last_mut() {
            previous.field.previous = Some(slot);
        }
        records.push(CopyField {
            slot,
            bucket,
            field: Field {
                owner,
                hash,
                key: bytes,
                value: field.value,
                previous: None,
                next: first,
            },
        });
        next = field.next;
    }
    if records.len() != source.count {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    Ok(records)
}
