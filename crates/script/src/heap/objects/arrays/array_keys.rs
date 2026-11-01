// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::{array_operation, array_resolution};
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::{
    ArrayId, ArrayKey, ArrayResult, HeapWorkBudget, ObjectHeap, ObjectHeapError,
    ObjectHeapErrorKind,
};

impl ObjectHeap {
    /// Borrows keys in current sibling order without copying payloads or owning data.
    /// The returned vector's logical rows count against operation scratch. Borrowing
    /// the heap prevents mutation/collection while its key slices are in use.
    /// # Errors
    /// Invalid references/link/key invariants, allocation or work/scratch limits
    /// publish no partial key list and leave the graph unchanged.
    pub fn array_keys(
        &self,
        data: ArrayId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<Vec<ArrayKey<'_>>>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let (owner, _) = array_resolution::data(self, data, work)?;
            let ObjectState::Occupied { first, fields } = self.objects[owner].state else {
                return Err(ObjectHeapErrorKind::InvalidState);
            };
            let scratch = (fields as u64)
                .checked_mul(size_of::<ArrayKey<'_>>() as u64)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            work.scratch(scratch)?;
            work.charge(fields as u64)?;
            let mut keys = Vec::new();
            keys.try_reserve_exact(fields)
                .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
            let (mut next, mut previous) = (first, None);
            while let Some(index) = next {
                work.charge(1)?;
                if keys.len() >= fields {
                    return Err(ObjectHeapErrorKind::InvalidState);
                }
                let Some(FieldSlot::Occupied(field)) = self.fields.get(index) else {
                    return Err(ObjectHeapErrorKind::InvalidState);
                };
                if field.owner != owner || field.previous != previous {
                    return Err(ObjectHeapErrorKind::InvalidState);
                }
                keys.push(FieldKey::decode_array(&field.key)?);
                previous = Some(index);
                next = field.next;
            }
            if keys.len() != fields {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            Ok(keys)
        })
    }
}
