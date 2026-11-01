// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::array_resolution;
use crate::heap::objects::field_index::Lookup;
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_error;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ArrayId, ArrayKey, ObjectHeap, ObjectHeapErrorKind, ObjectResource, StringHeap};

pub(super) fn find(
    heap: &ObjectHeap,
    data: ArrayId,
    key: ArrayKey<'_>,
    strings: &StringHeap,
    work: &mut ObjectWork,
) -> Result<(usize, u64, Lookup), ObjectHeapErrorKind> {
    let (owner, _) = array_resolution::data(heap, data, work)?;
    work.charge(1)?;
    if strings.owner != heap.string_owner {
        return Err(ObjectHeapErrorKind::ForeignStringHeap);
    }
    let key = FieldKey::Array(key);
    let length = key.len()? as u64;
    object_error::check(
        ObjectResource::SingleKeyBytes,
        length,
        heap.budget.max_single_key_bytes,
    )?;
    work.charge(length)?;
    let hash = key.hash(owner);
    let found = heap
        .index
        .lookup_key(&heap.fields, owner, hash, key, work)?;
    Ok((owner, hash, found))
}
