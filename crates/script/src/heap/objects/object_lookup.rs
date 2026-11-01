// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::execution::KernelValue;
use crate::heap::StringHeap;
use crate::heap::objects::field_index::Lookup;
use crate::heap::objects::object_error;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{ObjectHeap, ObjectHeapErrorKind, ObjectId, ObjectResource};

pub(super) fn find(
    heap: &ObjectHeap,
    id: ObjectId,
    key: &[u8],
    strings: &StringHeap,
    work: &mut ObjectWork,
) -> Result<(usize, u64, Lookup), ObjectHeapErrorKind> {
    work.charge(2)?;
    let owner = heap.resolve_struct(id)?;
    if strings.owner != heap.string_owner {
        return Err(ObjectHeapErrorKind::ForeignStringHeap);
    }
    object_error::check(
        ObjectResource::SingleKeyBytes,
        key.len() as u64,
        heap.budget.max_single_key_bytes,
    )?;
    work.charge(key.len() as u64)?;
    // This is a private, toolchain-pinned index hash, not a network/content digest.
    // Every collision probe is separately charged, independent of hash quality.
    let mut hasher = DefaultHasher::new();
    owner.hash(&mut hasher);
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let lookup = heap.index.lookup(&heap.fields, owner, hash, key, work)?;
    Ok((owner, hash, lookup))
}

pub(super) fn validate_value(
    heap: &ObjectHeap,
    value: KernelValue,
    strings: &StringHeap,
    work: &mut ObjectWork,
) -> Result<(), ObjectHeapErrorKind> {
    work.charge(1)?;
    match value {
        KernelValue::Array(_) => return Err(ObjectHeapErrorKind::ManagedArrayFieldUnsupported),
        KernelValue::Object(id) => {
            heap.resolve_struct(id)?;
        }
        KernelValue::String(id) | KernelValue::LocalizedString(id) => {
            strings.get(id).map_err(ObjectHeapErrorKind::StringHeap)?;
        }
        KernelValue::Undefined
        | KernelValue::Number(_)
        | KernelValue::Vector(_)
        | KernelValue::Function(_) => {}
    }
    Ok(())
}
