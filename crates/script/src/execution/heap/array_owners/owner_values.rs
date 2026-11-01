// SPDX-License-Identifier: AGPL-3.0-or-later
use super::array_frame::{HEAP_RELEASE_STEPS, VALUE_CLEANUP};
use crate::execution::heap::KernelArrayMutation;
use crate::execution::heap::kernel_object_error;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ArrayValueId, HeapWorkBudget};

impl KernelHeapContext<'_> {
    pub(in crate::execution) fn mutate_owned_array(
        &mut self,
        frame: usize,
        value: ArrayValueId,
        index: KernelValue,
        operation: KernelArrayMutation,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<(), KernelErrorKind> {
        operation.validate()?;
        self.owned_array_slot(frame, value)?;
        self.select_unique_array(value, roots.clone())?;
        self.mutate_array(value, index, operation, roots)
    }

    pub(in crate::execution) fn copy_host_array(
        &mut self,
        frame: usize,
        value: ArrayValueId,
    ) -> Result<ArrayValueId, KernelErrorKind> {
        self.prepare_array_value(frame)?;
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_deref_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .copy_array_value(value, budget);
        let result = match result {
            Ok(result) => result,
            Err(error) => return Err(kernel_object_error::record(self, error, 0)?),
        };
        self.publish_array_value(frame, result)
    }

    pub(in crate::execution) fn copy_owned_array(
        &mut self,
        frame: usize,
        value: ArrayValueId,
    ) -> Result<ArrayValueId, KernelErrorKind> {
        self.owned_array_slot(frame, value)?;
        self.copy_host_array(frame, value)
    }

    pub(in crate::execution) fn release_owned_array(
        &mut self,
        frame: usize,
        value: ArrayValueId,
    ) -> Result<(), KernelErrorKind> {
        let slot = self.owned_array_slot(frame, value)?;
        self.release_array_owner(value)?;
        self.array_owners.frames[frame].values.remove(slot);
        Ok(())
    }

    pub(super) fn release_array_owner(
        &mut self,
        value: ArrayValueId,
    ) -> Result<(), KernelErrorKind> {
        self.array_owners.consume_cleanup(VALUE_CLEANUP)?;
        let result = self
            .objects
            .as_deref_mut()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .release_array_value(
                value,
                HeapWorkBudget {
                    max_steps: HEAP_RELEASE_STEPS,
                    max_scratch_bytes: 0,
                },
            );
        if let Err(error) = result {
            self.array_owners.poisoned = true;
            return Err(KernelErrorKind::ObjectHeap(error.kind));
        }
        self.array_owners.live = self
            .array_owners
            .live
            .checked_sub(1)
            .ok_or(KernelErrorKind::ArrayOwnershipInvariant)?;
        Ok(())
    }

    pub(in crate::execution) fn move_owned_array(
        &mut self,
        from: usize,
        to: usize,
        value: ArrayValueId,
    ) -> Result<(), KernelErrorKind> {
        let source = self.owned_array_slot(from, value)?;
        if from == to {
            return Ok(());
        }
        self.prepare_array_frame(to, 1)?;
        let target = self.new_array_owner_slot(to, value)?;
        self.charge(2)?;
        self.array_owners.frames[from].values.remove(source);
        self.array_owners.frames[to].values.insert(target, value);
        Ok(())
    }
}
