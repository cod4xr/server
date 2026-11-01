// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{kernel_heap_collection, kernel_object_error};
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ObjectHeapErrorKind, ObjectId, ObjectResource};

impl KernelHeapContext<'_> {
    pub(super) fn field(
        &mut self,
        id: ObjectId,
        key: &[u8],
    ) -> Result<KernelValue, KernelErrorKind> {
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_ref()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .get(id, key, self.heap, budget);
        match result {
            Ok(result) => {
                self.record(result.usage, 0)?;
                Ok(result.value.unwrap_or(KernelValue::Undefined))
            }
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }

    pub(super) fn store_field(
        &mut self,
        id: ObjectId,
        key: &[u8],
        value: KernelValue,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<(), KernelErrorKind> {
        match self.attempt_field(id, key, value) {
            Ok(()) => return Ok(()),
            Err(KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::BudgetExceeded {
                resource:
                    ObjectResource::LiveFields
                    | ObjectResource::KeyBytes
                    | ObjectResource::FieldSlots
                    | ObjectResource::IndexBuckets
                    | ObjectResource::MetadataBytes,
                ..
            })) => {}
            Err(kind) => return Err(kind),
        }
        kernel_heap_collection::collect(self, roots)?;
        self.attempt_field(id, key, value)
    }

    fn attempt_field(
        &mut self,
        id: ObjectId,
        key: &[u8],
        value: KernelValue,
    ) -> Result<(), KernelErrorKind> {
        let budget = self.remaining(0)?;
        let objects = self
            .objects
            .as_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?;
        let result = if matches!(value, KernelValue::Undefined) {
            objects.remove(id, key, self.heap, budget)
        } else {
            objects.put(id, key, value, self.heap, budget)
        };
        match result {
            Ok(result) => self.record(result.usage, 0),
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }
}
