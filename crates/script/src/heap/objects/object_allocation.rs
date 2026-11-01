// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::HeapWorkBudget;
use crate::heap::objects::object_slot::{ObjectSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{
    ObjectAllocation, ObjectHeap, ObjectHeapError, ObjectHeapErrorKind, ObjectId,
};

pub(super) fn allocate(
    heap: &mut ObjectHeap,
    budget: HeapWorkBudget,
) -> Result<ObjectAllocation, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    let id = insert(heap, &mut work).map_err(|kind| work.error(kind))?;
    Ok(ObjectAllocation {
        id,
        usage: work.usage(),
    })
}

fn insert(heap: &mut ObjectHeap, work: &mut ObjectWork) -> Result<ObjectId, ObjectHeapErrorKind> {
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
    }
    let usage = usage.admit(heap.budget)?;
    if heap.free_object.is_none() {
        if heap.objects.len() == heap.objects.capacity() {
            work.charge(heap.objects.len() as u64)?;
        }
        heap.objects
            .try_reserve(1)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        heap.objects.push(ObjectSlot {
            generation,
            state: ObjectState::Occupied {
                first: None,
                fields: 0,
            },
        });
    } else {
        // The slot and its state were validated before any mutation; the vector cannot change here.
        heap.objects[index].state = ObjectState::Occupied {
            first: None,
            fields: 0,
        };
        heap.free_object = next;
    }
    heap.usage = usage;
    Ok(ObjectId {
        owner: heap.owner,
        slot: index,
        generation,
    })
}
