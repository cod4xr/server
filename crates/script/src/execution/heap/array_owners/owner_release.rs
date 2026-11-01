// SPDX-License-Identifier: AGPL-3.0-or-later
use super::array_frame::{KernelArrayOwners, ROW_CLEANUP};
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};

impl KernelHeapContext<'_> {
    pub(in crate::execution) fn return_array_frame(
        &mut self,
        frame: usize,
        value: KernelValue,
        parent: Option<usize>,
    ) -> Result<(), KernelErrorKind> {
        let KernelValue::Array(id) = value else {
            return self.close_array_frame(frame);
        };
        if self.array_owners.pending.is_some() {
            return Err(KernelErrorKind::ArrayOwnershipInvariant);
        }
        let source = self.owned_array_slot(frame, id)?;
        let target = if let Some(parent) = parent {
            if parent == frame {
                return Err(KernelErrorKind::ArrayOwnershipInvariant);
            }
            self.prepare_array_frame(parent, 1)?;
            Some((parent, self.new_array_owner_slot(parent, id)?))
        } else {
            None
        };
        // Resource-dependent admission/probes precede detachment and cleanup.
        self.charge(2)?;
        self.array_owners.frames[frame].values.remove(source);
        self.array_owners.pending = Some(id);
        self.close_array_frame(frame)?;
        if let Some((parent, slot)) = target {
            self.array_owners.frames[parent].values.insert(slot, id);
        } else {
            self.array_owners.live = self
                .array_owners
                .live
                .checked_sub(1)
                .ok_or(KernelErrorKind::ArrayOwnershipInvariant)?;
        }
        self.array_owners.pending = None;
        Ok(())
    }

    pub(super) fn close_array_frame(&mut self, frame: usize) -> Result<(), KernelErrorKind> {
        let Some(row) = self.array_owners.frames.get(frame) else {
            return Ok(());
        };
        if row.scan_credit == 0 {
            return Ok(());
        }
        let credit = row.scan_credit;
        self.array_owners.consume_cleanup(credit)?;
        let capacity = self.array_owners.frames[frame].values.capacity();
        let mut failure = None;
        for index in 0..capacity {
            if let Some(value) = self.array_owners.frames[frame].values.values[index].take()
                && let Err(error) = self.release_array_owner(value)
            {
                failure.get_or_insert(error);
            }
        }
        let row = &mut self.array_owners.frames[frame];
        row.values.used.fill(false);
        row.values.len = 0;
        row.values.filled = 0;
        row.scan_credit = 0;
        failure.map_or(Ok(()), Err)
    }

    pub(in crate::execution) fn finish_array_owners(&mut self) -> Result<(), KernelErrorKind> {
        let mut failure = None;
        if let Some(value) = self.array_owners.pending.take()
            && let Err(error) = self.release_array_owner(value)
        {
            failure.get_or_insert(error);
        }
        for index in (0..self.array_owners.frames.len()).rev() {
            if let Err(error) = self.close_array_frame(index) {
                failure.get_or_insert(error);
            }
        }
        let cleanup = (self.array_owners.frames.len() as u64)
            .checked_mul(ROW_CLEANUP)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        self.array_owners.consume_cleanup(cleanup)?;
        let poisoned =
            self.array_owners.poisoned || self.array_owners.live != 0 || failure.is_some();
        if poisoned {
            failure.get_or_insert(KernelErrorKind::ArrayOwnershipInvariant);
        }
        self.array_owners = KernelArrayOwners {
            poisoned,
            ..KernelArrayOwners::default()
        };
        failure.map_or(Ok(()), Err)
    }
}
