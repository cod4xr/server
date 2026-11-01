// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{kernel_heap_collection, kernel_object_error};
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ArrayValueId, ObjectHeapErrorKind, ObjectResource};

impl KernelHeapContext<'_> {
    pub(super) fn select_unique_array(
        &mut self,
        value: ArrayValueId,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<(), KernelErrorKind> {
        match self.attempt_unique_array(value) {
            Ok(()) => return Ok(()),
            Err(KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::BudgetExceeded {
                resource:
                    ObjectResource::LiveObjects
                    | ObjectResource::ObjectSlots
                    | ObjectResource::LiveFields
                    | ObjectResource::KeyBytes
                    | ObjectResource::FieldSlots
                    | ObjectResource::IndexBuckets
                    | ObjectResource::MetadataBytes,
                ..
            })) => {}
            Err(kind) => return Err(kind),
        }
        kernel_heap_collection::collect(self, roots)?;
        self.attempt_unique_array(value)
    }

    fn attempt_unique_array(&mut self, value: ArrayValueId) -> Result<(), KernelErrorKind> {
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_deref_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .make_array_unique(value, self.heap, budget);
        match result {
            Ok(result) => self.record(result.usage, 0),
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }
}
