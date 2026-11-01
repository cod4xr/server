// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::objects::object_slot::FieldSlot;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{
    ObjectFieldResult, ObjectHeap, ObjectHeapError, ObjectHeapErrorKind, ObjectId,
};
use crate::heap::objects::{field_insertion, field_removal, object_lookup};
use crate::heap::{HeapWorkBudget, StringHeap};

pub(super) fn get(
    heap: &ObjectHeap,
    id: ObjectId,
    key: &[u8],
    strings: &StringHeap,
    budget: HeapWorkBudget,
) -> Result<ObjectFieldResult, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    let result = (|| {
        let (_, _, found) = object_lookup::find(heap, id, key, strings, &mut work)?;
        let value = if let Some(index) = found.field {
            let Some(FieldSlot::Occupied(field)) = heap.fields.get(index) else {
                return Err(ObjectHeapErrorKind::InvalidState);
            };
            object_lookup::validate_value(heap, field.value, strings, &mut work)?;
            Some(field.value)
        } else {
            None
        };
        Ok(value)
    })();
    result
        .map(|value| ObjectFieldResult {
            value,
            usage: work.usage(),
        })
        .map_err(|kind| work.error(kind))
}

pub(super) fn put(
    heap: &mut ObjectHeap,
    id: ObjectId,
    key: &[u8],
    value: KernelValue,
    strings: &StringHeap,
    budget: HeapWorkBudget,
) -> Result<ObjectFieldResult, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    let result = (|| {
        let (owner, hash, found) = object_lookup::find(heap, id, key, strings, &mut work)?;
        object_lookup::validate_value(heap, value, strings, &mut work)?;
        if let Some(index) = found.field {
            let Some(FieldSlot::Occupied(field)) = heap.fields.get_mut(index) else {
                return Err(ObjectHeapErrorKind::InvalidState);
            };
            return Ok(Some(std::mem::replace(&mut field.value, value)));
        }
        field_insertion::insert(heap, owner, hash, key, value, found.bucket, &mut work)?;
        Ok(None)
    })();
    result
        .map(|value| ObjectFieldResult {
            value,
            usage: work.usage(),
        })
        .map_err(|kind| work.error(kind))
}

pub(super) fn remove(
    heap: &mut ObjectHeap,
    id: ObjectId,
    key: &[u8],
    strings: &StringHeap,
    budget: HeapWorkBudget,
) -> Result<ObjectFieldResult, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    let result = (|| {
        let (owner, _, found) = object_lookup::find(heap, id, key, strings, &mut work)?;
        if let Some(index) = found.field {
            let bucket = found.bucket.ok_or(ObjectHeapErrorKind::InvalidState)?;
            field_removal::remove(heap, owner, index, bucket, &mut work).map(Some)
        } else {
            Ok(None)
        }
    })();
    result
        .map(|value| ObjectFieldResult {
            value,
            usage: work.usage(),
        })
        .map_err(|kind| work.error(kind))
}
