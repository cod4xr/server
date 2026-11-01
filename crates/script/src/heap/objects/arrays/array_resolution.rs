// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::{ArrayId, ArrayValueId, ArrayValueState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind, ObjectId};

pub(super) fn data(
    heap: &ObjectHeap,
    id: ArrayId,
    work: &mut ObjectWork,
) -> Result<(usize, u64), ObjectHeapErrorKind> {
    work.charge(1)?;
    let index = heap.resolve(id.object)?;
    let metadata = heap
        .arrays
        .metadata
        .get(index)
        .ok_or(ObjectHeapErrorKind::ExpectedArray)?;
    if !metadata.is_array {
        return Err(ObjectHeapErrorKind::ExpectedArray);
    }
    Ok((index, metadata.owners))
}

pub(super) struct ArrayValueResolution {
    pub(super) slot: usize,
    pub(super) data: ObjectId,
    pub(super) data_index: usize,
    pub(super) owners: u64,
}

pub(super) fn value(
    heap: &ObjectHeap,
    id: ArrayValueId,
    work: &mut ObjectWork,
) -> Result<ArrayValueResolution, ObjectHeapErrorKind> {
    work.charge(1)?;
    if heap.arrays.owner != Some(id.owner) {
        return Err(ObjectHeapErrorKind::ForeignArrayValue);
    }
    let slot = heap
        .arrays
        .values
        .get(id.slot)
        .ok_or(ObjectHeapErrorKind::StaleArrayValue)?;
    if slot.generation != id.generation {
        return Err(ObjectHeapErrorKind::StaleArrayValue);
    }
    let ArrayValueState::Occupied(target) = slot.state else {
        return Err(ObjectHeapErrorKind::StaleArrayValue);
    };
    let (index, owners) = data(heap, ArrayId { object: target }, work)?;
    if owners == 0 {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    Ok(ArrayValueResolution {
        slot: id.slot,
        data: target,
        data_index: index,
        owners,
    })
}
