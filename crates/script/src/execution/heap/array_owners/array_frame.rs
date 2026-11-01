// SPDX-License-Identifier: AGPL-3.0-or-later
use super::owner_table::OwnerTable;
use crate::execution::KernelErrorKind;
use crate::heap::ArrayValueId;

pub(super) const VALUE_CLEANUP: u64 = 5;
pub(super) const HEAP_RELEASE_STEPS: u64 = 3;
pub(super) const ROW_CLEANUP: u64 = 2;

#[derive(Default)]
pub(super) struct KernelArrayFrame {
    pub(super) values: OwnerTable,
    pub(super) scan_credit: u64,
}

/// Tracks only invocation-owned locations; ordinary/host heap rows are never scanned.
#[derive(Default)]
pub(in crate::execution::heap) struct KernelArrayOwners {
    pub(super) frames: Vec<KernelArrayFrame>,
    pub(in crate::execution::heap) bytes: u64,
    pub(super) live: u64,
    pub(super) reserved: u64,
    pub(super) consumed: u64,
    pub(super) poisoned: bool,
    pub(super) pending: Option<ArrayValueId>,
}

impl KernelArrayOwners {
    pub(in crate::execution::heap) fn ready(&self) -> bool {
        !self.poisoned && self.live == 0 && self.frames.is_empty() && self.pending.is_none()
    }
    pub(super) fn reserve_cleanup(&mut self, count: u64) -> Result<(), KernelErrorKind> {
        self.reserved = self
            .reserved
            .checked_add(count)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        Ok(())
    }

    pub(super) fn consume_cleanup(&mut self, count: u64) -> Result<(), KernelErrorKind> {
        let next = self
            .consumed
            .checked_add(count)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        if next > self.reserved {
            return Err(KernelErrorKind::ArrayOwnershipInvariant);
        }
        self.consumed = next;
        Ok(())
    }
}
