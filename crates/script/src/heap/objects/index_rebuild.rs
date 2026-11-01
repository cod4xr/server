// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::ObjectHeapErrorKind;
use crate::heap::objects::field_index::{Bucket, FieldIndex};
use crate::heap::objects::object_slot::{FieldSlot, ObjectSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;

pub(super) fn stage(
    length: usize,
    fields: &[FieldSlot],
    objects: &[ObjectSlot],
    work: &mut ObjectWork,
) -> Result<FieldIndex, ObjectHeapErrorKind> {
    if !length.is_power_of_two() {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    let bytes = (length as u64)
        .checked_mul(size_of::<Bucket>() as u64)
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    work.scratch(bytes)?;
    work.charge(length as u64)?;
    let mut buckets = Vec::new();
    buckets
        .try_reserve_exact(length)
        .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
    buckets.resize(length, Bucket::Empty);
    let mut index = FieldIndex {
        buckets,
        live: 0,
        filled: 0,
    };
    for (slot, field) in fields.iter().enumerate() {
        work.charge(1)?;
        if let FieldSlot::Occupied(field) = field {
            if !matches!(
                objects.get(field.owner),
                Some(ObjectSlot {
                    state: ObjectState::Occupied { .. },
                    ..
                })
            ) {
                return Err(ObjectHeapErrorKind::InvalidState);
            }
            let bucket = index.empty_position(field.hash, work)?;
            index.insert(bucket, slot)?;
        }
    }
    Ok(index)
}
