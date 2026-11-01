// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests isolate final populated-copy resource admission from the admitted source state."
)]
use crate::array_copy_support::{graph, unchanged};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind, ObjectResource};

#[test]
fn populated_copy_admits_exact_final_resources_and_every_one_less_limit_is_atomic() {
    let mut sample = graph(4, 0, ObjectHeapBudget::default());
    sample
        .heap
        .make_array_unique(sample.selected, &sample.strings, HeapWorkBudget::default())
        .expect("Measure final resources");
    let usage = sample.heap.usage();
    let exact = ObjectHeapBudget {
        max_live_objects: usage.live_objects,
        max_object_slots: usage.object_slots,
        max_live_fields: usage.live_fields,
        max_field_slots: usage.field_slots,
        max_key_bytes: usage.key_bytes,
        max_index_buckets: usage.index_buckets,
        max_metadata_bytes: usage.metadata_bytes,
        max_array_values: usage.live_array_values,
        max_array_value_slots: usage.array_value_slots,
        ..ObjectHeapBudget::default()
    };
    for resource in [
        None,
        Some(ObjectResource::LiveObjects),
        Some(ObjectResource::ObjectSlots),
        Some(ObjectResource::LiveFields),
        Some(ObjectResource::FieldSlots),
        Some(ObjectResource::KeyBytes),
        Some(ObjectResource::IndexBuckets),
        Some(ObjectResource::MetadataBytes),
    ] {
        let mut limits = exact;
        match resource {
            Some(ObjectResource::LiveObjects) => limits.max_live_objects -= 1,
            Some(ObjectResource::ObjectSlots) => limits.max_object_slots -= 1,
            Some(ObjectResource::LiveFields) => limits.max_live_fields -= 1,
            Some(ObjectResource::FieldSlots) => limits.max_field_slots -= 1,
            Some(ObjectResource::KeyBytes) => limits.max_key_bytes -= 1,
            Some(ObjectResource::IndexBuckets) => limits.max_index_buckets -= 1,
            Some(ObjectResource::MetadataBytes) => limits.max_metadata_bytes -= 1,
            _ => {}
        }
        let mut state = graph(4, 0, limits);
        let before = state.heap.usage();
        let result =
            state
                .heap
                .make_array_unique(state.selected, &state.strings, HeapWorkBudget::default());
        if let Some(resource) = resource {
            assert!(
                matches!(result.expect_err("One less final resource").kind, ObjectHeapErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
            );
            unchanged(&state, before);
        } else {
            result.expect("Exact final resources");
            assert_eq!(state.heap.usage(), usage);
        }
    }
}
