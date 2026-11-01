// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::{
    ArrayCowOutcome, ArrayMetadata, ArrayValueState, array_operation, array_resolution,
};
use crate::heap::objects::object_allocation_plan::ObjectAllocationPlan;
use crate::heap::objects::object_slot::ObjectState;
use crate::heap::{
    ArrayResult, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapError, ObjectHeapErrorKind,
};

impl ObjectHeap {
    /// Selects unique data for a managed empty-array value, detaching aliases.
    ///
    /// This complete empty-data profile does not implement populated/nested copy,
    /// cycle-alias markers or source arrays. Populated data must use a future
    /// populated-copy operation; raw shared-struct field APIs still reject arrays.
    /// # Errors
    /// Invalid references, nonempty data, limits/overflow or allocation
    /// publish no detachment/counter change. Spare capacity may grow.
    pub fn make_empty_array_unique(
        &mut self,
        value: ArrayValueId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<ArrayCowOutcome>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let resolved = array_resolution::value(self, value, work)?;
            if !matches!(
                self.objects[resolved.data_index].state,
                ObjectState::Occupied {
                    fields: 0,
                    first: None
                }
            ) {
                return Err(ObjectHeapErrorKind::ExpectedEmptyArray);
            }
            if resolved.owners == 1 {
                return Ok(ArrayCowOutcome::Unchanged);
            }
            let owners = resolved
                .owners
                .checked_sub(1)
                .ok_or(ObjectHeapErrorKind::InvalidState)?;
            let mut plan = ObjectAllocationPlan::new(self, work)?;
            plan.usage.live_arrays = plan
                .usage
                .live_arrays
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            plan.usage.array_metadata_rows = plan.usage.object_slots;
            plan.reserve(self, work)?;
            let data = plan.publish(self);
            self.arrays.metadata[data.slot] = ArrayMetadata {
                is_array: true,
                owners: 1,
            };
            self.arrays.metadata[resolved.data_index].owners = owners;
            self.arrays.values[resolved.slot].state = ArrayValueState::Occupied(data);
            Ok(ArrayCowOutcome::Detached)
        })
    }
}
