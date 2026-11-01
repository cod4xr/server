// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::execution::KernelErrorKind;
use crate::heap::ArrayValueId;

/// Narrow fixed-key index of VM locations, independent of unrelated heap slots.
/// Explicit logical bucket lengths allow admission before allocation and metered probes.
#[derive(Default)]
pub(super) struct OwnerTable {
    pub(super) values: Vec<Option<ArrayValueId>>,
    pub(super) used: Vec<bool>,
    pub(super) len: usize,
    pub(super) filled: usize,
}

#[derive(Debug)]
pub(super) struct OwnerLookup {
    pub(super) present: bool,
    pub(super) slot: usize,
}

impl OwnerTable {
    pub(super) fn capacity(&self) -> usize {
        self.values.len()
    }

    pub(super) fn bytes(capacity: usize) -> Result<u64, KernelErrorKind> {
        (capacity as u64)
            .checked_mul((size_of::<Option<ArrayValueId>>() + size_of::<bool>()) as u64)
            .ok_or(KernelErrorKind::SizeOverflow)
    }

    pub(super) fn replacement(&self, additional: usize) -> Result<Option<usize>, KernelErrorKind> {
        let count = self
            .len
            .checked_add(additional)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        let filled = self
            .filled
            .checked_add(additional)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        let capacity = self.capacity();
        let threshold = capacity - capacity / 4;
        if count > threshold {
            return count
                .checked_mul(2)
                .and_then(|count| count.max(8).checked_next_power_of_two())
                .map(Some)
                .ok_or(KernelErrorKind::SizeOverflow);
        }
        Ok((filled > threshold).then_some(capacity))
    }

    pub(super) fn allocate(capacity: usize) -> Result<Self, KernelErrorKind> {
        if !capacity.is_power_of_two() || capacity < 8 {
            return Err(KernelErrorKind::ArrayOwnershipInvariant);
        }
        let mut values = Vec::new();
        let mut used = Vec::new();
        values
            .try_reserve_exact(capacity)
            .map_err(|_| KernelErrorKind::AllocationFailed)?;
        used.try_reserve_exact(capacity)
            .map_err(|_| KernelErrorKind::AllocationFailed)?;
        values.resize(capacity, None);
        used.resize(capacity, false);
        Ok(Self {
            values,
            used,
            len: 0,
            filled: 0,
        })
    }

    pub(super) fn lookup(
        &self,
        value: ArrayValueId,
        mut charge: impl FnMut(u64) -> Result<(), KernelErrorKind>,
    ) -> Result<OwnerLookup, KernelErrorKind> {
        charge(1)?;
        let mask = self
            .capacity()
            .checked_sub(1)
            .ok_or(KernelErrorKind::ArrayOwnershipInvariant)?;
        let mut hasher = DefaultHasher::new();
        value.hash_location(&mut hasher);
        let slot = usize::try_from(hasher.finish() & mask as u64)
            .map_err(|_| KernelErrorKind::SizeOverflow)?;
        self.lookup_from(value, slot, charge)
    }

    fn lookup_from(
        &self,
        value: ArrayValueId,
        mut slot: usize,
        mut charge: impl FnMut(u64) -> Result<(), KernelErrorKind>,
    ) -> Result<OwnerLookup, KernelErrorKind> {
        let mask = self
            .capacity()
            .checked_sub(1)
            .ok_or(KernelErrorKind::ArrayOwnershipInvariant)?;
        let mut deleted = None;
        for _ in 0..self.capacity() {
            charge(1)?;
            if self.values[slot] == Some(value) {
                return Ok(OwnerLookup {
                    present: true,
                    slot,
                });
            }
            if self.values[slot].is_none() {
                if !self.used[slot] {
                    return Ok(OwnerLookup {
                        present: false,
                        slot: deleted.unwrap_or(slot),
                    });
                }
                deleted.get_or_insert(slot);
            }
            slot = (slot + 1) & mask;
        }
        deleted
            .map(|slot| OwnerLookup {
                present: false,
                slot,
            })
            .ok_or(KernelErrorKind::ArrayOwnershipInvariant)
    }

    pub(super) fn insert(&mut self, slot: usize, value: ArrayValueId) {
        // Callers admitted the count and obtained an absent slot without further
        // intervening table mutation. Publication itself cannot fail or allocate.
        self.values[slot] = Some(value);
        self.len += 1;
        self.filled += usize::from(!self.used[slot]);
        self.used[slot] = true;
    }

    pub(super) fn remove(&mut self, slot: usize) {
        self.values[slot] = None;
        self.len -= 1;
    }
}

#[cfg(test)]
#[path = "owner_table_tests.rs"]
mod tests;
