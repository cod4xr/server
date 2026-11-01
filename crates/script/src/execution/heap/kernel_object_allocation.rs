// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{kernel_heap_collection, kernel_object_error};
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ObjectHeapErrorKind, ObjectId, ObjectResource};

impl KernelHeapContext<'_> {
    pub(super) fn allocate_object(
        &mut self,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<ObjectId, KernelErrorKind> {
        match self.attempt_object() {
            Ok(id) => return Ok(id),
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
        self.attempt_object()
    }

    fn attempt_object(&mut self) -> Result<ObjectId, KernelErrorKind> {
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .allocate(budget);
        match result {
            Ok(result) => {
                self.record(result.usage, 0)?;
                Ok(result.id)
            }
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }
}
