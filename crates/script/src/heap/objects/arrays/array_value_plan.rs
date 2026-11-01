// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::heap_identity::HeapIdentity;
use crate::heap::objects::arrays::{ArrayValueId, ArrayValueSlot, ArrayValueState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind, ObjectHeapUsage, ObjectId};

pub(super) struct ArrayValuePlan {
    index: usize,
    generation: u64,
    next: Option<usize>,
    append: bool,
}

impl ArrayValuePlan {
    pub(super) fn new(
        heap: &ObjectHeap,
        usage: &mut ObjectHeapUsage,
        work: &mut ObjectWork,
    ) -> Result<Self, ObjectHeapErrorKind> {
        work.charge(1)?;
        let index = heap.arrays.free_value.unwrap_or(heap.arrays.values.len());
        let (generation, next) = match heap.arrays.free_value {
            Some(index) => match heap.arrays.values.get(index) {
                Some(ArrayValueSlot {
                    generation,
                    state: ArrayValueState::Vacant { next },
                }) => (*generation, *next),
                _ => return Err(ObjectHeapErrorKind::InvalidState),
            },
            None => (1, None),
        };
        usage.live_array_values = usage
            .live_array_values
            .checked_add(1)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        if heap.arrays.free_value.is_none() {
            usage.array_value_slots = usage
                .array_value_slots
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        }
        Ok(Self {
            index,
            generation,
            next,
            append: heap.arrays.free_value.is_none(),
        })
    }

    pub(super) fn reserve(
        &self,
        heap: &mut ObjectHeap,
        work: &mut ObjectWork,
    ) -> Result<HeapIdentity, ObjectHeapErrorKind> {
        if self.append {
            if heap.arrays.values.len() == heap.arrays.values.capacity() {
                work.charge(heap.arrays.values.len() as u64)?;
            }
            heap.arrays
                .values
                .try_reserve(1)
                .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        }
        if let Some(owner) = heap.arrays.owner {
            Ok(owner)
        } else {
            HeapIdentity::allocate().map_err(|_| ObjectHeapErrorKind::IdentityExhausted)
        }
    }

    pub(super) fn publish(
        self,
        heap: &mut ObjectHeap,
        target: ObjectId,
        owner: HeapIdentity,
    ) -> ArrayValueId {
        let state = ArrayValueState::Occupied(target);
        if self.append {
            heap.arrays.values.push(ArrayValueSlot {
                generation: self.generation,
                state,
            });
        } else {
            heap.arrays.values[self.index].state = state;
            heap.arrays.free_value = self.next;
        }
        heap.arrays.owner = Some(owner);
        ArrayValueId {
            owner,
            slot: self.index,
            generation: self.generation,
        }
    }
}
