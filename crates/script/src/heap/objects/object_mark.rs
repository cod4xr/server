// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{ObjectHeap, ObjectHeapErrorKind, ObjectId};

pub(super) fn trace(
    heap: &ObjectHeap,
    roots: impl Iterator<Item = Option<ObjectId>> + Clone,
    work: &mut ObjectWork,
) -> Result<Vec<bool>, ObjectHeapErrorKind> {
    for id in roots.clone() {
        work.charge(1)?;
        if let Some(id) = id {
            heap.resolve(id)?;
        }
    }
    let count = heap.objects.len();
    let live =
        usize::try_from(heap.usage.live_objects).map_err(|_| ObjectHeapErrorKind::SizeOverflow)?;
    let bytes = (live as u64)
        .checked_mul(size_of::<usize>() as u64)
        .and_then(|queue| queue.checked_add(count as u64))
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    work.scratch(bytes)?;
    work.charge(count as u64)?;
    let mut flags = Vec::new();
    flags
        .try_reserve_exact(count)
        .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
    flags.resize(count, false);
    let mut queue = Vec::new();
    queue
        .try_reserve_exact(live)
        .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
    for id in roots {
        work.charge(1)?;
        if let Some(id) = id {
            enqueue(heap.resolve(id)?, &mut flags, &mut queue, live)?;
        }
    }
    let mut cursor = 0;
    while cursor < queue.len() {
        work.charge(1)?;
        let owner = queue[cursor];
        cursor += 1;
        let (mut next, expected) = match heap.objects.get(owner).map(|slot| &slot.state) {
            Some(ObjectState::Occupied { first, fields }) => (*first, *fields),
            _ => return Err(ObjectHeapErrorKind::InvalidState),
        };
        let mut visited = 0;
        let mut previous = None;
        while let Some(index) = next {
            work.charge(1)?;
            if visited >= expected {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            visited += 1;
            let Some(FieldSlot::Occupied(field)) = heap.fields.get(index) else {
                return Err(ObjectHeapErrorKind::InvalidState);
            };
            if field.owner != owner || field.previous != previous {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            match field.value {
                KernelValue::Array(_) => {
                    return Err(ObjectHeapErrorKind::ManagedArrayFieldUnsupported);
                }
                KernelValue::Object(id) => {
                    work.charge(1)?;
                    enqueue(heap.resolve(id)?, &mut flags, &mut queue, live)?;
                }
                KernelValue::Undefined
                | KernelValue::Number(_)
                | KernelValue::Vector(_)
                | KernelValue::Function(_)
                | KernelValue::String(_)
                | KernelValue::LocalizedString(_) => {}
            }
            previous = Some(index);
            next = field.next;
        }
        if visited != expected {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
    }
    Ok(flags)
}

fn enqueue(
    index: usize,
    flags: &mut [bool],
    queue: &mut Vec<usize>,
    maximum: usize,
) -> Result<(), ObjectHeapErrorKind> {
    let flag = flags
        .get_mut(index)
        .ok_or(ObjectHeapErrorKind::InvalidState)?;
    if !*flag {
        if queue.len() >= maximum {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        *flag = true;
        queue.push(index);
    }
    Ok(())
}
