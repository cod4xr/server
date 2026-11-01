// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::array_resolution::ArrayValueResolution;
use crate::heap::objects::arrays::copy::copy_fields::{self, CopyField};
use crate::heap::objects::arrays::copy::copy_index::CopyIndex;
use crate::heap::objects::arrays::copy::copy_layout::CopyLayout;
use crate::heap::objects::arrays::{ArrayMetadata, ArrayValueState};
use crate::heap::objects::object_allocation_plan::ObjectAllocationPlan;
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind, StringHeap};

pub(in crate::heap::objects::arrays) struct CopyPlan {
    parent: ObjectAllocationPlan,
    fields: Vec<CopyField>,
    index: CopyIndex,
    next_free: Option<usize>,
    source: usize,
    owners: u64,
    value_slot: usize,
}

impl CopyPlan {
    pub(in crate::heap::objects::arrays) fn stage(
        heap: &mut ObjectHeap,
        value: &ArrayValueResolution,
        strings: &StringHeap,
        work: &mut ObjectWork,
    ) -> Result<Self, ObjectHeapErrorKind> {
        let owners = value
            .owners
            .checked_sub(1)
            .filter(|owners| *owners > 0)
            .ok_or(ObjectHeapErrorKind::InvalidState)?;
        let mut layout = CopyLayout::new(heap, value.data_index, strings, work)?;
        // Prepay flat record cleanup and publication before owning staged keys.
        work.charge(
            (layout.source.count as u64)
                .checked_mul(2)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?,
        )?;
        let mut index = CopyIndex::new(
            heap,
            layout.index,
            &layout.source,
            layout.parent.id.slot,
            layout.records_bytes,
            work,
        )?;
        let fields = copy_fields::stage(
            heap,
            layout.parent.id.slot,
            &layout.source,
            &mut index,
            work,
        )?;
        if layout.append > 0 {
            let length = heap
                .fields
                .len()
                .checked_add(layout.append)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            if length > heap.fields.capacity() {
                work.charge(heap.fields.len() as u64)?;
            }
            heap.fields
                .try_reserve(layout.append)
                .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        }
        layout.parent.reserve(heap, work)?;
        work.charge(1)?;
        Ok(Self {
            parent: layout.parent,
            fields,
            index,
            next_free: layout.next_free,
            source: value.data_index,
            owners,
            value_slot: value.slot,
        })
    }

    pub(in crate::heap::objects::arrays) fn publish(self, heap: &mut ObjectHeap) {
        let first = self.fields.last().map(|field| field.slot);
        let count = self.fields.len();
        let data = self.parent.publish(heap);
        self.index.publish(heap, &self.fields);
        for record in self.fields {
            let field = FieldSlot::Occupied(record.field);
            if record.slot == heap.fields.len() {
                heap.fields.push(field);
            } else {
                heap.fields[record.slot] = field;
            }
        }
        heap.free_field = self.next_free;
        heap.objects[data.slot].state = ObjectState::Occupied {
            first,
            fields: count,
        };
        heap.arrays.metadata[data.slot] = ArrayMetadata {
            is_array: true,
            owners: 1,
        };
        heap.arrays.metadata[self.source].owners = self.owners;
        heap.arrays.values[self.value_slot].state = ArrayValueState::Occupied(data);
    }
}
