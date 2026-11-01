// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{kernel_heap_collection, kernel_object_error};
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ArrayResult, ArrayValueId, ObjectHeapErrorKind, ObjectResource};

impl KernelHeapContext<'_> {
    pub(super) fn allocate_empty_array(
        &mut self,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<ArrayResult<ArrayValueId>, KernelErrorKind> {
        match self.attempt_empty_array() {
            Ok(result) => return Ok(result),
            Err(KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::BudgetExceeded {
                resource:
                    ObjectResource::LiveObjects
                    | ObjectResource::ObjectSlots
                    | ObjectResource::MetadataBytes,
                ..
            })) => {}
            Err(kind) => return Err(kind),
        }
        kernel_heap_collection::collect(self, roots)?;
        self.attempt_empty_array()
    }

    fn attempt_empty_array(&mut self) -> Result<ArrayResult<ArrayValueId>, KernelErrorKind> {
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_deref_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .new_array_value(budget);
        match result {
            // The owner registry must cover success before recording usage.
            Ok(result) => Ok(result),
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }
}
