// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::array_value_plan::ArrayValuePlan;
use crate::heap::objects::arrays::{ArrayMetadata, array_operation};
use crate::heap::objects::object_allocation_plan::ObjectAllocationPlan;
use crate::heap::{
    ArrayResult, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapError, ObjectHeapErrorKind,
};

impl ObjectHeap {
    /// Creates empty array data and one explicitly owned value location.
    ///
    /// The location is a collection root until [`Self::release_array_value`] is called.
    /// Copying its handle creates no additional owner. This storage component
    /// does not activate source arrays or the array VM. Indexed storage uses the
    /// separate checked unique-entry APIs.
    /// # Errors
    /// Limits, overflow/identity exhaustion and fallible allocation publish no
    /// value/data node. Spare arena capacity may grow outside logical counters.
    pub fn new_array_value(
        &mut self,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<ArrayValueId>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let mut object = ObjectAllocationPlan::new(self, work)?;
            object.usage.live_arrays = object
                .usage
                .live_arrays
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            object.usage.array_metadata_rows = object.usage.object_slots;
            let value = ArrayValuePlan::new(self, &mut object.usage, work)?;
            object.reserve(self, work)?;
            let owner = value.reserve(self, work)?;
            let data = object.publish(self);
            self.arrays.metadata[data.slot] = ArrayMetadata {
                is_array: true,
                owners: 1,
            };
            Ok(value.publish(self, data, owner))
        })
    }
}
