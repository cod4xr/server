// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::ArrayValueState;
use crate::heap::objects::arrays::array_value_plan::ArrayValuePlan;
use crate::heap::objects::arrays::{array_operation, array_resolution};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{
    ArrayId, ArrayResult, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapError,
    ObjectHeapErrorKind, ObjectId,
};

impl ObjectHeap {
    /// Copies a managed value into a distinct owned location sharing the same data.
    /// Both locations remain roots until explicitly released; raw handle copies
    /// alone do not change ownership. No data contents are copied here.
    /// # Errors
    /// Invalid references, limits/overflow and allocation leave existing values,
    /// counters and logical usage unchanged; spare capacity may grow.
    pub fn copy_array_value(
        &mut self,
        value: ArrayValueId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<ArrayValueId>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let resolved = array_resolution::value(self, value, work)?;
            retain(
                self,
                resolved.data,
                resolved.data_index,
                resolved.owners,
                work,
            )
        })
    }

    /// Creates an explicitly owned location from a still-valid nonowning snapshot.
    /// The caller must do this before any collection that could reclaim its data.
    /// # Errors
    /// Invalid data, limits/overflow and allocation publish no location or counter
    /// change; spare capacity may grow outside logical usage.
    pub fn retain_array(
        &mut self,
        data: ArrayId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<ArrayValueId>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let (index, owners) = array_resolution::data(self, data, work)?;
            retain(self, data.object, index, owners, work)
        })
    }

    /// Releases one managed location and invalidates every copy of its handle.
    /// Array data is reclaimed by explicit collection, not by this decrement.
    /// # Errors
    /// Invalid references, exhausted work or private counter inconsistencies leave
    /// values/usage unchanged. Exhausted generations retire without reuse.
    pub fn release_array_value(
        &mut self,
        value: ArrayValueId,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<()>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let resolved = array_resolution::value(self, value, work)?;
            work.charge(1)?;
            let owners = resolved
                .owners
                .checked_sub(1)
                .ok_or(ObjectHeapErrorKind::InvalidState)?;
            let live = self
                .usage
                .live_array_values
                .checked_sub(1)
                .ok_or(ObjectHeapErrorKind::InvalidState)?;
            let slot = &mut self.arrays.values[resolved.slot];
            if let Some(generation) = slot.generation.checked_add(1) {
                slot.generation = generation;
                slot.state = ArrayValueState::Vacant {
                    next: self.arrays.free_value,
                };
                self.arrays.free_value = Some(resolved.slot);
            } else {
                slot.state = ArrayValueState::Retired;
            }
            self.arrays.metadata[resolved.data_index].owners = owners;
            self.usage.live_array_values = live;
            Ok(())
        })
    }
}

fn retain(
    heap: &mut ObjectHeap,
    data: ObjectId,
    index: usize,
    owners: u64,
    work: &mut ObjectWork,
) -> Result<ArrayValueId, ObjectHeapErrorKind> {
    let owners = owners
        .checked_add(1)
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    let mut usage = heap.usage;
    let plan = ArrayValuePlan::new(heap, &mut usage, work)?;
    let usage = usage.admit(heap.budget)?;
    let owner = plan.reserve(heap, work)?;
    heap.arrays.metadata[index].owners = owners;
    let value = plan.publish(heap, data, owner);
    heap.usage = usage;
    Ok(value)
}
