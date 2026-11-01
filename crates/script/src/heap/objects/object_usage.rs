// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::HeapWorkUsage;
use crate::heap::objects::field_index::Bucket;
use crate::heap::objects::object_error;
use crate::heap::objects::object_slot::{FieldSlot, ObjectSlot};
use crate::heap::objects::{ObjectHeapBudget, ObjectHeapErrorKind, ObjectId, ObjectResource};

/// Retained logical object storage, excluding allocator rounding/spare capacity/RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ObjectHeapUsage {
    /// Live shared objects.
    pub live_objects: u64,
    /// Live fields.
    pub live_fields: u64,
    /// Owned live field-key bytes.
    pub key_bytes: u64,
    /// Retained object slots, including vacant/retired slots.
    pub object_slots: u64,
    /// Retained field slots, including vacant slots.
    pub field_slots: u64,
    /// Retained field-index bucket records.
    pub index_buckets: u64,
    /// Logical complete arena/index record bytes.
    pub metadata_bytes: u64,
}

/// Complete successful object allocation.
#[derive(Clone, Copy, Debug)]
pub struct ObjectAllocation {
    /// Checked nonowning identity requiring explicit rooting.
    pub id: ObjectId,
    /// Charged operation work/scratch.
    pub usage: HeapWorkUsage,
}

/// Copied field result; None means absent, distinct from stored Undefined.
#[derive(Clone, Copy, Debug)]
pub struct ObjectFieldResult {
    /// Read/previous/removed copied value. It does not create an ownership lease.
    pub value: Option<KernelValue>,
    /// Charged operation work/scratch.
    pub usage: HeapWorkUsage,
}

/// Successful explicit object-graph collection, without collecting string payloads.
#[derive(Clone, Copy, Debug)]
pub struct ObjectCollection {
    /// Unreachable objects reclaimed.
    pub objects: u64,
    /// Unreachable fields reclaimed.
    pub fields: u64,
    /// Owned key payload bytes reclaimed.
    pub key_bytes: u64,
    /// Charged visits and peak scratch.
    pub usage: HeapWorkUsage,
}

/// String collection including object fields and supplied frame/cache/host roots.
#[derive(Clone, Copy, Debug)]
pub struct ObjectStringCollection {
    /// String values reclaimed by the bound string store.
    pub strings: u64,
    /// String payload bytes reclaimed.
    pub payload_bytes: u64,
    /// Combined root discovery and string collection work/scratch.
    pub usage: HeapWorkUsage,
}

impl ObjectHeapUsage {
    pub(super) fn admit(mut self, budget: ObjectHeapBudget) -> Result<Self, ObjectHeapErrorKind> {
        let objects = self
            .object_slots
            .checked_mul(size_of::<ObjectSlot>() as u64);
        let fields = self.field_slots.checked_mul(size_of::<FieldSlot>() as u64);
        let buckets = self.index_buckets.checked_mul(size_of::<Bucket>() as u64);
        self.metadata_bytes = objects
            .zip(fields)
            .zip(buckets)
            .and_then(|((objects, fields), buckets)| {
                objects.checked_add(fields)?.checked_add(buckets)
            })
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        for (resource, requested, maximum) in [
            (
                ObjectResource::LiveObjects,
                self.live_objects,
                budget.max_live_objects,
            ),
            (
                ObjectResource::LiveFields,
                self.live_fields,
                budget.max_live_fields,
            ),
            (
                ObjectResource::KeyBytes,
                self.key_bytes,
                budget.max_key_bytes,
            ),
            (
                ObjectResource::ObjectSlots,
                self.object_slots,
                budget.max_object_slots,
            ),
            (
                ObjectResource::FieldSlots,
                self.field_slots,
                budget.max_field_slots,
            ),
            (
                ObjectResource::IndexBuckets,
                self.index_buckets,
                budget.max_index_buckets,
            ),
            (
                ObjectResource::MetadataBytes,
                self.metadata_bytes,
                budget.max_metadata_bytes,
            ),
        ] {
            object_error::check(resource, requested, maximum)?;
        }
        Ok(self)
    }
}
