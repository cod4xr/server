// SPDX-License-Identifier: AGPL-3.0-or-later

/// Configurable shared-object storage, independent of native packed pool widths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectHeapBudget {
    /// Live shared objects, including objects with no fields.
    pub max_live_objects: u64,
    /// Live named fields across the complete store.
    pub max_live_fields: u64,
    /// Sum of owned live field-key bytes.
    pub max_key_bytes: u64,
    /// Retained object slots, including vacant/retired generations.
    pub max_object_slots: u64,
    /// Explicitly owned array value locations, independent of array data count.
    pub max_array_values: u64,
    /// Retained array value slots, including vacant/retired generations.
    pub max_array_value_slots: u64,
    /// Retained field slots, including vacant slots.
    pub max_field_slots: u64,
    /// Retained field-index bucket records.
    pub max_index_buckets: u64,
    /// Logical arena/index records; excludes payload/allocator/spare capacity/RSS.
    pub max_metadata_bytes: u64,
    /// Complete byte length of any supplied field key.
    pub max_single_key_bytes: u64,
}

impl Default for ObjectHeapBudget {
    fn default() -> Self {
        Self {
            max_live_objects: 1_000_000,
            max_live_fields: 1_000_000,
            max_key_bytes: 64 * 1024 * 1024,
            max_object_slots: 1_000_000,
            max_array_values: 1_000_000,
            max_array_value_slots: 1_000_000,
            max_field_slots: 1_000_000,
            max_index_buckets: 4 * 1024 * 1024,
            max_metadata_bytes: 256 * 1024 * 1024,
            max_single_key_bytes: 1024 * 1024,
        }
    }
}
