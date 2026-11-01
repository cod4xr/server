// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind};

pub(super) fn walk(
    heap: &ObjectHeap,
    owner: usize,
    work: &mut ObjectWork,
    mut reference: impl FnMut(usize) -> Result<(), ObjectHeapErrorKind>,
) -> Result<usize, ObjectHeapErrorKind> {
    let Some(ObjectState::Occupied { first, fields }) =
        heap.objects.get(owner).map(|slot| &slot.state)
    else {
        return Err(ObjectHeapErrorKind::InvalidState);
    };
    let (mut next, mut visited, mut previous) = (*first, 0, None);
    while let Some(index) = next {
        work.charge(1)?;
        if visited >= *fields {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        let Some(FieldSlot::Occupied(field)) = heap.fields.get(index) else {
            return Err(ObjectHeapErrorKind::InvalidState);
        };
        if field.owner != owner || field.previous != previous {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        match field.value {
            KernelValue::Array(_) => return Err(ObjectHeapErrorKind::ManagedArrayFieldUnsupported),
            KernelValue::Object(id) => {
                work.charge(1)?;
                reference(heap.resolve(id)?)?;
            }
            KernelValue::Undefined
            | KernelValue::Number(_)
            | KernelValue::Vector(_)
            | KernelValue::Function(_)
            | KernelValue::String(_)
            | KernelValue::LocalizedString(_) => {}
        }
        visited += 1;
        previous = Some(index);
        next = field.next;
    }
    if visited != *fields {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    Ok(visited)
}
