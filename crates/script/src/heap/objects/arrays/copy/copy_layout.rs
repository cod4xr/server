// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::copy::copy_fields::CopyField;
use crate::heap::objects::arrays::copy::copy_index::CopyIndexLayout;
use crate::heap::objects::arrays::copy::copy_source::{self, CopySource};
use crate::heap::objects::object_allocation_plan::ObjectAllocationPlan;
use crate::heap::objects::object_slot::FieldSlot;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind, StringHeap};

pub(super) struct CopyLayout {
    pub(super) parent: ObjectAllocationPlan,
    pub(super) source: CopySource,
    pub(super) next_free: Option<usize>,
    pub(super) append: usize,
    pub(super) index: CopyIndexLayout,
    pub(super) records_bytes: u64,
}

impl CopyLayout {
    pub(super) fn new(
        heap: &ObjectHeap,
        owner: usize,
        strings: &StringHeap,
        work: &mut ObjectWork,
    ) -> Result<Self, ObjectHeapErrorKind> {
        let mut parent = ObjectAllocationPlan::new(heap, work)?;
        let source = copy_source::inspect(heap, owner, strings, work)?;
        let mut free = heap.free_field;
        let mut reused = 0;
        while reused < source.count {
            let Some(slot) = free else {
                break;
            };
            work.charge(1)?;
            let Some(FieldSlot::Vacant { next }) = heap.fields.get(slot) else {
                return Err(ObjectHeapErrorKind::InvalidState);
            };
            free = *next;
            reused += 1;
        }
        let append = source.count - reused;
        let slots = heap
            .fields
            .len()
            .checked_add(append)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        let index = CopyIndexLayout::new(heap, source.count)?;
        let usage = &mut parent.usage;
        usage.live_arrays = usage
            .live_arrays
            .checked_add(1)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        usage.live_fields = usage
            .live_fields
            .checked_add(source.count as u64)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        usage.key_bytes = usage
            .key_bytes
            .checked_add(source.bytes)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        usage.field_slots = slots as u64;
        usage.index_buckets = index.length as u64;
        usage.array_metadata_rows = usage.object_slots;
        parent.usage = parent.usage.admit(heap.budget)?;
        let records_bytes = (source.count as u64)
            .checked_mul(size_of::<CopyField>() as u64)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        Ok(Self {
            parent,
            source,
            next_free: free,
            append,
            index,
            records_bytes,
        })
    }
}
