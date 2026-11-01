// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{kernel_array_keys, kernel_heap_collection, kernel_object_error};
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ArrayValueId, ObjectHeapErrorKind, ObjectResource};

#[derive(Clone, Copy)]
pub(in crate::execution) enum KernelArrayMutation {
    Assign(KernelValue),
    Clear,
}

impl KernelArrayMutation {
    pub(in crate::execution) fn validate(self) -> Result<(), KernelErrorKind> {
        if matches!(self, Self::Assign(KernelValue::Array(_))) {
            return Err(KernelErrorKind::ObjectHeap(
                ObjectHeapErrorKind::ManagedArrayFieldUnsupported,
            ));
        }
        Ok(())
    }
}

impl KernelHeapContext<'_> {
    pub(super) fn mutate_array(
        &mut self,
        value: ArrayValueId,
        index: KernelValue,
        operation: KernelArrayMutation,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<(), KernelErrorKind> {
        if matches!(
            operation,
            KernelArrayMutation::Assign(KernelValue::Undefined)
        ) {
            self.ensure_array_child(value, index, roots)?;
            return self.attempt_array_mutation(value, index, KernelArrayMutation::Clear);
        }
        match self.attempt_array_mutation(value, index, operation) {
            Ok(()) => return Ok(()),
            Err(kind)
                if field_pressure(kind) && matches!(operation, KernelArrayMutation::Assign(_)) => {}
            Err(kind) => return Err(kind),
        }
        kernel_heap_collection::collect(self, roots)?;
        self.attempt_array_mutation(value, index, operation)
    }

    fn ensure_array_child(
        &mut self,
        value: ArrayValueId,
        index: KernelValue,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<(), KernelErrorKind> {
        match self.attempt_array_child(value, index) {
            Ok(()) => return Ok(()),
            Err(kind) if field_pressure(kind) => {}
            Err(kind) => return Err(kind),
        }
        kernel_heap_collection::collect(self, roots)?;
        self.attempt_array_child(value, index)
    }

    fn attempt_array_child(
        &mut self,
        value: ArrayValueId,
        index: KernelValue,
    ) -> Result<(), KernelErrorKind> {
        self.charge(1)?;
        let key = kernel_array_keys::legacy_index(index, self.heap)?;
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_deref_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .array_ensure_unique(value, key, self.heap, budget);
        match result {
            Ok(result) => self.record(result.usage, 0),
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }

    fn attempt_array_mutation(
        &mut self,
        value: ArrayValueId,
        index: KernelValue,
        operation: KernelArrayMutation,
    ) -> Result<(), KernelErrorKind> {
        self.charge(1)?;
        let key = kernel_array_keys::legacy_index(index, self.heap)?;
        let budget = self.remaining(0)?;
        let objects = self
            .objects
            .as_deref_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?;
        let result = match operation {
            KernelArrayMutation::Assign(element) => {
                objects.array_put_unique(value, key, element, self.heap, budget)
            }
            KernelArrayMutation::Clear => {
                objects.array_remove_unique(value, key, self.heap, budget)
            }
        };
        match result {
            Ok(result) => self.record(result.usage, 0),
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }
}

fn field_pressure(kind: KernelErrorKind) -> bool {
    matches!(
        kind,
        KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::LiveFields
                | ObjectResource::KeyBytes
                | ObjectResource::FieldSlots
                | ObjectResource::IndexBuckets
                | ObjectResource::MetadataBytes,
            ..
        })
    )
}
