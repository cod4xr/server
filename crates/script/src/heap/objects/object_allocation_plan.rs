// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::ArrayMetadata;
use crate::heap::objects::object_slot::{ObjectSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind, ObjectHeapUsage, ObjectId};

pub(super) struct ObjectAllocationPlan {
    pub(super) id: ObjectId,
    pub(super) usage: ObjectHeapUsage,
    next: Option<usize>,
    append: bool,
    metadata_rows: usize,
}

impl ObjectAllocationPlan {
    pub(super) fn new(
        heap: &ObjectHeap,
        work: &mut ObjectWork,
    ) -> Result<Self, ObjectHeapErrorKind> {
        work.charge(1)?;
        let index = heap.free_object.unwrap_or(heap.objects.len());
        let (generation, next) = match heap.free_object {
            Some(index) => match heap.objects.get(index) {
                Some(ObjectSlot {
                    generation,
                    state: ObjectState::Vacant { next },
                }) => (*generation, *next),
                _ => return Err(ObjectHeapErrorKind::InvalidState),
            },
            None => (1, None),
        };
        if heap.arrays.is_array(index) {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        let mut usage = heap.usage;
        usage.live_objects = usage
            .live_objects
            .checked_add(1)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        if heap.free_object.is_none() {
            usage.object_slots = usage
                .object_slots
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            if !heap.arrays.metadata.is_empty() {
                usage.array_metadata_rows = usage
                    .array_metadata_rows
                    .checked_add(1)
                    .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            }
        }
        Ok(Self {
            id: ObjectId {
                owner: heap.owner,
                slot: index,
                generation,
            },
            usage,
            next,
            append: heap.free_object.is_none(),
            metadata_rows: heap.arrays.metadata.len(),
        })
    }

    pub(super) fn reserve(
        &mut self,
        heap: &mut ObjectHeap,
        work: &mut ObjectWork,
    ) -> Result<(), ObjectHeapErrorKind> {
        self.usage = self.usage.admit(heap.budget)?;
        if self.append {
            if heap.objects.len() == heap.objects.capacity() {
                work.charge(heap.objects.len() as u64)?;
            }
            heap.objects
                .try_reserve(1)
                .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        }
        let count = usize::try_from(self.usage.array_metadata_rows)
            .map_err(|_| ObjectHeapErrorKind::SizeOverflow)?;
        let extra = count
            .checked_sub(heap.arrays.metadata.len())
            .ok_or(ObjectHeapErrorKind::InvalidState)?;
        if extra > 0 {
            work.charge(extra as u64)?;
            if count > heap.arrays.metadata.capacity() {
                work.charge(heap.arrays.metadata.len() as u64)?;
            }
            heap.arrays
                .metadata
                .try_reserve(extra)
                .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        }
        self.metadata_rows = count;
        Ok(())
    }

    pub(super) fn publish(self, heap: &mut ObjectHeap) -> ObjectId {
        let state = ObjectState::Occupied {
            first: None,
            fields: 0,
        };
        if self.append {
            heap.objects.push(ObjectSlot {
                generation: self.id.generation,
                state,
            });
        } else {
            heap.objects[self.id.slot].state = state;
            heap.free_object = self.next;
        }
        // reserve validated the count conversion and requested all rows before
        // publication. Every producer supplies counts derived from arena lengths.
        heap.arrays
            .metadata
            .resize(self.metadata_rows, ArrayMetadata::default());
        heap.usage = self.usage;
        self.id
    }
}
