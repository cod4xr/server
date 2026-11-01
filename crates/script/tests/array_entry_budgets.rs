// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests isolate exact resource admission and private failures for typed array entries."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind, ObjectResource,
};

#[test]
fn array_entry_resource_limits_include_complete_encoded_keys_and_leave_no_partial_insert() {
    let budget = HeapWorkBudget::default();
    let (strings, mut sample) = stores(ObjectHeapBudget::default());
    let value = sample.new_array_value(budget).expect("Sample").value;
    sample
        .array_put_unique(
            value,
            ArrayKey::Index(u64::MAX),
            number(7),
            &strings,
            budget,
        )
        .expect("Measure");
    let exact = ObjectHeapBudget {
        max_live_fields: 1,
        max_field_slots: 1,
        max_key_bytes: 9,
        max_single_key_bytes: 9,
        max_index_buckets: 8,
        max_metadata_bytes: sample.usage().metadata_bytes,
        ..ObjectHeapBudget::default()
    };
    for resource in [
        None,
        Some(ObjectResource::LiveFields),
        Some(ObjectResource::FieldSlots),
        Some(ObjectResource::KeyBytes),
        Some(ObjectResource::SingleKeyBytes),
        Some(ObjectResource::IndexBuckets),
        Some(ObjectResource::MetadataBytes),
    ] {
        let mut limits = exact;
        match resource {
            Some(ObjectResource::LiveFields) => limits.max_live_fields -= 1,
            Some(ObjectResource::FieldSlots) => limits.max_field_slots -= 1,
            Some(ObjectResource::KeyBytes) => limits.max_key_bytes -= 1,
            Some(ObjectResource::SingleKeyBytes) => limits.max_single_key_bytes -= 1,
            Some(ObjectResource::IndexBuckets) => limits.max_index_buckets -= 1,
            Some(ObjectResource::MetadataBytes) => limits.max_metadata_bytes -= 1,
            _ => {}
        }
        let (strings, mut heap) = stores(limits);
        let value = heap.new_array_value(budget).expect("Base still fits").value;
        let before = heap.usage();
        let result = heap.array_put_unique(
            value,
            ArrayKey::Index(u64::MAX),
            number(7),
            &strings,
            budget,
        );
        if let Some(resource) = resource {
            assert!(
                matches!(result.expect_err("One less resource").kind, ObjectHeapErrorKind::BudgetExceeded {resource: actual, requested, maximum} if actual == resource && requested > maximum)
            );
            assert_eq!(heap.usage(), before);
        } else {
            assert_eq!(result.expect("All exact limits").value, None);
            assert_eq!(heap.usage(), sample.usage());
        }
    }
}
