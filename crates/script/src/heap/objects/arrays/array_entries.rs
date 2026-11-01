// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::objects::arrays::{array_lookup, array_operation, array_resolution};
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_slot::FieldSlot;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{field_insertion, field_removal, object_lookup};
use crate::heap::{
    ArrayId, ArrayKey, ArrayResult, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapError,
    ObjectHeapErrorKind, StringHeap,
};

impl ObjectHeap {
    /// Reads a checked entry without creating an owner or detaching data.
    /// Missing entries differ from stored Undefined. Values use the current
    /// primitive/shared-struct profile; nested array values remain unsupported.
    /// # Errors
    /// Invalid stores/references, key/work limits and private state publish no changes.
    pub fn array_get(
        &self,
        data: ArrayId,
        key: ArrayKey<'_>,
        strings: &StringHeap,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<Option<KernelValue>>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let (_, _, found) = array_lookup::find(self, data, key, strings, work)?;
            let Some(index) = found.field else {
                return Ok(None);
            };
            let Some(FieldSlot::Occupied(field)) = self.fields.get(index) else {
                return Err(ObjectHeapErrorKind::InvalidState);
            };
            object_lookup::validate_value(self, field.value, strings, work)?;
            Ok(Some(field.value))
        })
    }

    /// Inserts/replaces an entry through a UNIQUE managed location.
    ///
    /// Shared values reject before mutation; this operation does not select COW
    /// data. Keys own a lossless private tag plus payload (nine bytes for an index,
    /// one plus byte-key length), included in key quotas. Raw Undefined is stored;
    /// source Undefined assignment/deletion rules belong to future VM lowering.
    /// New keys prepend to sibling order; replacements preserve their position.
    /// # Errors
    /// Invalid/shared values, invalid references, limits, overflow or allocation
    /// preserve data, locations, counters and logical usage. Spare capacity may grow.
    pub fn array_put_unique(
        &mut self,
        value: ArrayValueId,
        key: ArrayKey<'_>,
        element: KernelValue,
        strings: &StringHeap,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<Option<KernelValue>>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let data = unique(self, value, work)?;
            let (owner, hash, found) = array_lookup::find(self, data, key, strings, work)?;
            object_lookup::validate_value(self, element, strings, work)?;
            if let Some(index) = found.field {
                let Some(FieldSlot::Occupied(field)) = self.fields.get_mut(index) else {
                    return Err(ObjectHeapErrorKind::InvalidState);
                };
                return Ok(Some(std::mem::replace(&mut field.value, element)));
            }
            field_insertion::insert(
                self,
                owner,
                hash,
                FieldKey::Array(key),
                element,
                found.bucket,
                work,
            )?;
            Ok(None)
        })
    }

    /// Removes a key through a UNIQUE managed location, without selecting COW data.
    /// The returned value is a nonowning snapshot; retain its ordinary object/
    /// string roots before later collection. Missing keys return None.
    /// # Errors
    /// Invalid/shared values, invalid stores/keys/references or work limits leave
    /// entries, identities and logical usage unchanged.
    pub fn array_remove_unique(
        &mut self,
        value: ArrayValueId,
        key: ArrayKey<'_>,
        strings: &StringHeap,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<Option<KernelValue>>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let data = unique(self, value, work)?;
            let (owner, _, found) = array_lookup::find(self, data, key, strings, work)?;
            match (found.field, found.bucket) {
                (Some(index), Some(bucket)) => {
                    field_removal::remove(self, owner, index, bucket, work).map(Some)
                }
                (None, _) => Ok(None),
                _ => Err(ObjectHeapErrorKind::InvalidState),
            }
        })
    }
}

fn unique(
    heap: &ObjectHeap,
    value: ArrayValueId,
    work: &mut ObjectWork,
) -> Result<ArrayId, ObjectHeapErrorKind> {
    let resolved = array_resolution::value(heap, value, work)?;
    if resolved.owners != 1 {
        return Err(ObjectHeapErrorKind::SharedArray);
    }
    Ok(ArrayId {
        object: resolved.data,
    })
}
