// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::{array_operation, array_resolution};
use crate::heap::objects::object_slot::ObjectState;
use crate::heap::{
    ArrayId, ArrayResult, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapError,
    ObjectHeapErrorKind,
};

impl ObjectHeap {
    /// Returns a checked nonowning data snapshot for one live owned value location.
    /// # Errors
    /// Rejects stale/foreign values/data and exhausted work without mutation.
    pub fn array_data(
        &self,
        value: ArrayValueId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<ArrayId>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            array_resolution::value(self, value, work).map(|resolved| ArrayId {
                object: resolved.data,
            })
        })
    }

    /// Returns the checked stored array entry count; current arrays are empty.
    /// This is metadata access, not source .size or native packed-size arithmetic.
    /// # Errors
    /// Rejects stale/foreign/wrong-kind data and exhausted work without mutation.
    pub fn array_len(
        &self,
        data: ArrayId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<u64>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let (index, _) = array_resolution::data(self, data, work)?;
            match self.objects[index].state {
                ObjectState::Occupied { fields, .. } => Ok(fields as u64),
                _ => Err(ObjectHeapErrorKind::InvalidState),
            }
        })
    }

    /// Returns explicitly owned value-location count, not native u16 excess refs.
    /// A valid unowned data snapshot may report zero before explicit collection.
    /// # Errors
    /// Rejects invalid data references and exhausted work without mutation.
    pub fn array_owner_count(
        &self,
        data: ArrayId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<u64>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            array_resolution::data(self, data, work).map(|(_, owners)| owners)
        })
    }
}
