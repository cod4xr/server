// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::copy::CopyPlan;
use crate::heap::objects::arrays::{array_operation, array_resolution};
use crate::heap::{
    ArrayCowOutcome, ArrayResult, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapError,
    ObjectHeapErrorKind, StringHeap,
};

impl ObjectHeap {
    /// Selects unique data, copying supported populated entries when shared.
    ///
    /// Copies reverse sibling order, own complete keys and preserve primitive/string
    /// values and ordinary struct identity. The selected location keeps its identity;
    /// other locations keep the original data. Current entries cannot contain nested
    /// arrays. Cycle-alias markers and general stored-array/logical lifetimes
    /// remain unimplemented.
    /// Unique selection performs no graph scan, copy or allocation.
    /// # Errors
    /// Invalid stores/references, private invariants, work/scratch/resource limits,
    /// overflow or allocation leave all data/locations/counters/logical usage
    /// unchanged. Spare arena capacity may grow. Publication cannot fail.
    pub fn make_array_unique(
        &mut self,
        value: ArrayValueId,
        strings: &StringHeap,
        budget: HeapWorkBudget,
    ) -> Result<ArrayResult<ArrayCowOutcome>, ObjectHeapError> {
        array_operation::run(budget, |work| {
            let value = array_resolution::value(self, value, work)?;
            work.charge(1)?;
            if strings.owner != self.string_owner {
                return Err(ObjectHeapErrorKind::ForeignStringHeap);
            }
            if value.owners == 1 {
                return Ok(ArrayCowOutcome::Unchanged);
            }
            CopyPlan::stage(self, &value, strings, work)?.publish(self);
            Ok(ArrayCowOutcome::Detached)
        })
    }
}
