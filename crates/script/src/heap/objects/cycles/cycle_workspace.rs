// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::object_mark;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind, ObjectId};

pub(super) struct CycleWorkspace {
    pub(super) live: Vec<bool>,
    pub(super) degrees: Vec<usize>,
    pub(super) targets: Vec<bool>,
    pub(super) queue: Vec<usize>,
    pub(super) maximum: usize,
    pub(super) scratch: u64,
}

impl CycleWorkspace {
    pub(super) fn new(
        heap: &ObjectHeap,
        roots: &[ObjectId],
        work: &mut ObjectWork,
    ) -> Result<Self, ObjectHeapErrorKind> {
        let live = object_mark::trace(heap, roots, work)?;
        let count = heap.objects.len();
        let maximum = usize::try_from(heap.usage.live_objects)
            .map_err(|_| ObjectHeapErrorKind::SizeOverflow)?;
        let scratch = (count as u64)
            .checked_mul((2 + size_of::<usize>()) as u64)
            .and_then(|bytes| {
                (maximum as u64)
                    .checked_mul(size_of::<usize>() as u64)
                    .and_then(|queue| bytes.checked_add(queue))
            })
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        work.scratch(scratch)?;
        work.charge(
            (count as u64)
                .checked_mul(2)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?,
        )?;
        let mut degrees = Vec::new();
        degrees
            .try_reserve_exact(count)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        degrees.resize(count, 0);
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(count)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        targets.resize(count, false);
        let mut queue = Vec::new();
        queue
            .try_reserve_exact(maximum)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        Ok(Self {
            live,
            degrees,
            targets,
            queue,
            maximum,
            scratch,
        })
    }

    pub(super) fn enqueue(&mut self, index: usize) -> Result<(), ObjectHeapErrorKind> {
        if self.queue.len() >= self.maximum {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        self.queue.push(index);
        Ok(())
    }

    pub(super) fn output(
        &self,
        heap: &ObjectHeap,
        work: &mut ObjectWork,
    ) -> Result<Vec<ObjectId>, ObjectHeapErrorKind> {
        work.charge(self.targets.len() as u64)?;
        let count = self.targets.iter().filter(|target| **target).count();
        let bytes = (count as u64)
            .checked_mul(size_of::<ObjectId>() as u64)
            .and_then(|output| self.scratch.checked_add(output))
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        work.scratch(bytes)?;
        work.charge(self.targets.len() as u64)?;
        work.charge(count as u64)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        for (index, target) in self.targets.iter().copied().enumerate() {
            if target {
                let slot = heap
                    .objects
                    .get(index)
                    .ok_or(ObjectHeapErrorKind::InvalidState)?;
                let id = ObjectId {
                    owner: heap.owner,
                    slot: index,
                    generation: slot.generation,
                };
                heap.resolve(id)?;
                output.push(id);
            }
        }
        Ok(output)
    }
}
