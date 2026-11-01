// SPDX-License-Identifier: AGPL-3.0-or-later
use super::owner_table::OwnerLookup;
use crate::execution::heap::kernel_heap_accounting;
use crate::execution::{KernelErrorKind, KernelHeapContext};
use crate::heap::ArrayValueId;

impl KernelHeapContext<'_> {
    pub(super) fn lookup_array_owner(
        &mut self,
        frame: usize,
        value: ArrayValueId,
    ) -> Result<OwnerLookup, KernelErrorKind> {
        let row = self
            .array_owners
            .frames
            .get(frame)
            .ok_or(KernelErrorKind::ArrayOwnershipInvariant)?;
        row.values.lookup(value, |count| {
            kernel_heap_accounting::charge(&mut self.usage, self.budget.max_steps, count)
        })
    }

    pub(super) fn owned_array_slot(
        &mut self,
        frame: usize,
        value: ArrayValueId,
    ) -> Result<usize, KernelErrorKind> {
        let found = self.lookup_array_owner(frame, value)?;
        if !found.present {
            return Err(KernelErrorKind::ArrayOwnershipInvariant);
        }
        Ok(found.slot)
    }

    pub(super) fn new_array_owner_slot(
        &mut self,
        frame: usize,
        value: ArrayValueId,
    ) -> Result<usize, KernelErrorKind> {
        let found = self.lookup_array_owner(frame, value)?;
        if found.present {
            return Err(KernelErrorKind::ArrayOwnershipInvariant);
        }
        Ok(found.slot)
    }
}
