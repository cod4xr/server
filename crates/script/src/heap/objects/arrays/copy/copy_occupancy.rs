// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::ObjectHeapErrorKind;
use crate::heap::objects::object_work::ObjectWork;

/// Scratch set of bucket positions changed by this copy, not a whole-index clone.
pub(super) struct CopyOccupancy {
    positions: Vec<Option<usize>>,
}

impl CopyOccupancy {
    pub(super) fn capacity(count: usize) -> Result<usize, ObjectHeapErrorKind> {
        count
            .checked_mul(2)
            .and_then(usize::checked_next_power_of_two)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)
    }

    pub(super) fn new(capacity: usize, work: &mut ObjectWork) -> Result<Self, ObjectHeapErrorKind> {
        if capacity == 0 || !capacity.is_power_of_two() {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        work.charge(capacity as u64)?;
        let mut positions = Vec::new();
        positions
            .try_reserve_exact(capacity)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        positions.resize(capacity, None);
        Ok(Self { positions })
    }

    pub(super) fn contains(
        &self,
        bucket: usize,
        work: &mut ObjectWork,
    ) -> Result<bool, ObjectHeapErrorKind> {
        let mut position = self.start(bucket);
        for _ in 0..self.positions.len() {
            work.charge(1)?;
            match self.positions[position] {
                Some(current) if current == bucket => return Ok(true),
                None => return Ok(false),
                Some(_) => position = (position + 1) & (self.positions.len() - 1),
            }
        }
        Err(ObjectHeapErrorKind::InvalidState)
    }

    pub(super) fn insert(
        &mut self,
        bucket: usize,
        work: &mut ObjectWork,
    ) -> Result<(), ObjectHeapErrorKind> {
        let mut position = self.start(bucket);
        for _ in 0..self.positions.len() {
            work.charge(1)?;
            match self.positions[position] {
                None => {
                    self.positions[position] = Some(bucket);
                    return Ok(());
                }
                Some(current) if current == bucket => {
                    return Err(ObjectHeapErrorKind::InvalidState);
                }
                Some(_) => position = (position + 1) & (self.positions.len() - 1),
            }
        }
        Err(ObjectHeapErrorKind::InvalidState)
    }

    fn start(&self, bucket: usize) -> usize {
        bucket.wrapping_mul(0x9e37_79b9) & (self.positions.len() - 1)
    }

    pub(super) fn clear(&mut self, work: &mut ObjectWork) -> Result<(), ObjectHeapErrorKind> {
        work.charge(self.positions.len() as u64)?;
        self.positions.fill(None);
        Ok(())
    }
}
