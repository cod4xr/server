// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::copy::copy_fields::CopyField;
use crate::heap::objects::arrays::copy::copy_occupancy::CopyOccupancy;
use crate::heap::objects::arrays::copy::copy_patch;
use crate::heap::objects::arrays::copy::copy_source::CopySource;
use crate::heap::objects::field_index::{Bucket, FieldIndex};
use crate::heap::objects::index_rebuild;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ObjectHeap, ObjectHeapErrorKind};

#[derive(Clone, Copy)]
pub(super) struct CopyIndexLayout {
    pub(super) length: usize,
    pub(super) count: usize,
    pub(super) live: usize,
    rebuild: bool,
}

impl CopyIndexLayout {
    pub(super) fn new(heap: &ObjectHeap, count: usize) -> Result<Self, ObjectHeapErrorKind> {
        let live = heap
            .index
            .live
            .checked_add(count)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        let old = heap.index.buckets.len();
        let mut length = old;
        if count > 0 {
            length = length.max(8);
            while live > length / 2 {
                length = length
                    .checked_mul(2)
                    .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            }
        }
        Ok(Self {
            length,
            count,
            live,
            rebuild: count > 0 && (old == 0 || length != old),
        })
    }
}

pub(super) enum CopyIndex {
    Unchanged,
    Rebuilt(FieldIndex),
    Patched {
        occupied: CopyOccupancy,
        live: usize,
        filled: usize,
    },
}

impl CopyIndex {
    pub(super) fn new(
        heap: &ObjectHeap,
        layout: CopyIndexLayout,
        source: &CopySource,
        owner: usize,
        records: u64,
        work: &mut ObjectWork,
    ) -> Result<Self, ObjectHeapErrorKind> {
        if layout.count == 0 {
            work.scratch(records)?;
            return Ok(Self::Unchanged);
        }
        if !layout.rebuild
            && let Some(index) = copy_patch::stage(heap, layout, source, owner, records, work)?
        {
            return Ok(index);
        }
        let scratch = (layout.length as u64)
            .checked_mul(size_of::<Bucket>() as u64)
            .and_then(|bytes| records.checked_add(bytes))
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        work.scratch(scratch)?;
        let index = index_rebuild::stage(layout.length, &heap.fields, &heap.objects, work)?;
        if index.live != heap.index.live {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        Ok(Self::Rebuilt(index))
    }

    pub(super) fn place(
        &mut self,
        heap: &ObjectHeap,
        hash: u64,
        slot: usize,
        work: &mut ObjectWork,
    ) -> Result<usize, ObjectHeapErrorKind> {
        match self {
            Self::Unchanged => Err(ObjectHeapErrorKind::InvalidState),
            Self::Rebuilt(index) => {
                let position = index.empty_position(hash, work)?;
                index.insert(position, slot)?;
                Ok(position)
            }
            Self::Patched {
                occupied, filled, ..
            } => {
                let mut position = heap.index.start(hash)?;
                for _ in 0..heap.index.buckets.len() {
                    work.charge(1)?;
                    if !occupied.contains(position, work)?
                        && !matches!(heap.index.buckets[position], Bucket::Occupied(_))
                    {
                        if matches!(heap.index.buckets[position], Bucket::Empty) {
                            *filled = filled
                                .checked_add(1)
                                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
                        }
                        occupied.insert(position, work)?;
                        return Ok(position);
                    }
                    position = (position + 1) & (heap.index.buckets.len() - 1);
                }
                Err(ObjectHeapErrorKind::InvalidState)
            }
        }
    }

    pub(super) fn publish(self, heap: &mut ObjectHeap, fields: &[CopyField]) {
        match self {
            Self::Unchanged => {}
            Self::Rebuilt(index) => heap.index = index,
            Self::Patched { live, filled, .. } => {
                for field in fields {
                    heap.index.buckets[field.bucket] = Bucket::Occupied(field.slot);
                }
                heap.index.live = live;
                heap.index.filled = filled;
            }
        }
    }
}
