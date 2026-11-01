// SPDX-License-Identifier: AGPL-3.0-or-later
use super::array_frame::{KernelArrayFrame, ROW_CLEANUP};
use super::owner_table::OwnerTable;
use crate::execution::{KernelErrorKind, KernelHeapContext};

impl KernelHeapContext<'_> {
    pub(in crate::execution) fn prepare_array_frame(
        &mut self,
        frame: usize,
        additional: usize,
    ) -> Result<(), KernelErrorKind> {
        if self.array_owners.poisoned {
            return Err(KernelErrorKind::ArrayOwnershipInvariant);
        }
        self.array_frame_rows(frame)?;
        let row = self
            .array_owners
            .frames
            .get(frame)
            .ok_or(KernelErrorKind::ArrayOwnershipInvariant)?;
        if let Some(capacity) = row.values.replacement(additional)? {
            self.stage_array_table(frame, capacity)?;
        } else if row.scan_credit == 0 {
            let credit = scan_credit(row.values.capacity())?;
            self.charge(credit)?;
            self.array_owners.reserve_cleanup(credit)?;
            self.array_owners.frames[frame].scan_credit = credit;
        }
        Ok(())
    }

    fn stage_array_table(&mut self, frame: usize, capacity: usize) -> Result<(), KernelErrorKind> {
        let bytes = OwnerTable::bytes(capacity)?;
        // Exact logical bucket lengths are known before either vector allocates.
        // The old table remains intact throughout fallible staging and rehashing.
        self.scratch(bytes)?;
        self.charge(
            (capacity as u64)
                .checked_mul(2)
                .ok_or(KernelErrorKind::SizeOverflow)?,
        )?;
        let mut staged = OwnerTable::allocate(capacity)?;
        let old_capacity = self.array_owners.frames[frame].values.capacity();
        for index in 0..old_capacity {
            self.charge(1)?;
            if let Some(value) = self.array_owners.frames[frame].values.values[index] {
                let found = staged.lookup(value, |count| self.charge(count))?;
                if found.present {
                    return Err(KernelErrorKind::ArrayOwnershipInvariant);
                }
                staged.insert(found.slot, value);
            }
        }
        let credit = scan_credit(capacity)?;
        let additional = credit.saturating_sub(self.array_owners.frames[frame].scan_credit);
        self.charge(additional)?;
        self.array_owners.reserve_cleanup(additional)?;
        let next_bytes = self
            .array_owners
            .bytes
            .checked_sub(OwnerTable::bytes(old_capacity)?)
            .and_then(|base| base.checked_add(bytes))
            .ok_or(KernelErrorKind::SizeOverflow)?;
        self.array_owners.frames[frame].values = staged;
        self.array_owners.frames[frame].scan_credit = credit;
        self.array_owners.bytes = next_bytes;
        Ok(())
    }

    fn array_frame_rows(&mut self, frame: usize) -> Result<(), KernelErrorKind> {
        let count = frame.checked_add(1).ok_or(KernelErrorKind::SizeOverflow)?;
        let current = self.array_owners.frames.len();
        if count <= current {
            return Ok(());
        }
        let additional = count - current;
        let bytes = (additional as u64)
            .checked_mul(size_of::<KernelArrayFrame>() as u64)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        let relocation = if count > self.array_owners.frames.capacity() {
            current as u64
        } else {
            0
        };
        let overlap = relocation
            .checked_mul(size_of::<KernelArrayFrame>() as u64)
            .and_then(|value| value.checked_add(bytes))
            .ok_or(KernelErrorKind::SizeOverflow)?;
        self.scratch(overlap)?;
        let cleanup = (additional as u64)
            .checked_mul(ROW_CLEANUP)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        let work = cleanup
            .checked_add(additional as u64)
            .and_then(|value| value.checked_add(relocation))
            .ok_or(KernelErrorKind::SizeOverflow)?;
        self.charge(work)?;
        self.array_owners.reserve_cleanup(cleanup)?;
        let next_bytes = self
            .array_owners
            .bytes
            .checked_add(bytes)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        self.array_owners
            .frames
            .try_reserve(additional)
            .map_err(|_| KernelErrorKind::AllocationFailed)?;
        self.array_owners
            .frames
            .resize_with(count, KernelArrayFrame::default);
        self.array_owners.bytes = next_bytes;
        Ok(())
    }
}

fn scan_credit(capacity: usize) -> Result<u64, KernelErrorKind> {
    (capacity as u64)
        .checked_add(1)
        .and_then(|value| value.checked_mul(2))
        .ok_or(KernelErrorKind::SizeOverflow)
}
