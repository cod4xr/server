// SPDX-License-Identifier: AGPL-3.0-or-later
use super::array_frame::VALUE_CLEANUP;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ArrayResult, ArrayValueId};

impl KernelHeapContext<'_> {
    pub(in crate::execution) fn construct_empty_array(
        &mut self,
        frame: usize,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<ArrayValueId, KernelErrorKind> {
        self.prepare_array_value(frame)?;
        let result = self.allocate_empty_array(roots)?;
        self.publish_array_value(frame, result)
    }

    pub(super) fn prepare_array_value(&mut self, frame: usize) -> Result<(), KernelErrorKind> {
        if self.array_owners.pending.is_some() {
            return Err(KernelErrorKind::ArrayOwnershipInvariant);
        }
        self.prepare_array_frame(frame, 1)?;
        self.charge(VALUE_CLEANUP)?;
        self.array_owners.reserve_cleanup(VALUE_CLEANUP)
    }

    pub(super) fn publish_array_value(
        &mut self,
        frame: usize,
        result: ArrayResult<ArrayValueId>,
    ) -> Result<ArrayValueId, KernelErrorKind> {
        // No fallible accounting or probes may precede fresh-owner coverage.
        self.array_owners.pending = Some(result.value);
        self.array_owners.live = self
            .array_owners
            .live
            .checked_add(1)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        self.record(result.usage, 0)?;
        let slot = self.new_array_owner_slot(frame, result.value)?;
        self.array_owners.frames[frame]
            .values
            .insert(slot, result.value);
        self.array_owners.pending = None;
        Ok(result.value)
    }
}
