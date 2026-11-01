// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests retain managed values/data beyond native ref and pool widths with checked release."
)]
use std::collections::HashSet;

use crate::object_heap_support::stores;
use cod4x_script::heap::{ArrayCowOutcome, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn seventy_thousand_owned_aliases_keep_wide_counts_and_detach_only_the_selected_value() {
    let (_, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let first = heap.new_array_value(budget).expect("Initial owner").value;
    let original = heap.array_data(first, budget).expect("Data").value;
    let mut values = vec![first];
    for _ in 1..70_000 {
        values.push(
            heap.copy_array_value(first, budget)
                .expect("Wide owned value arena")
                .value,
        );
    }
    assert_eq!(heap.usage().live_arrays, 1);
    assert_eq!(heap.usage().live_array_values, 70_000);
    assert_eq!(heap.usage().array_value_slots, 70_000);
    assert_eq!(
        heap.array_owner_count(original, budget)
            .expect("No u16 counter wrapping")
            .value,
        70_000
    );
    assert_eq!(
        heap.make_empty_array_unique(values[69_999], budget)
            .expect("Detach one location")
            .value,
        ArrayCowOutcome::Detached
    );
    assert_eq!(
        heap.array_owner_count(original, budget)
            .expect("Old aliases retained")
            .value,
        69_999
    );
    assert_eq!(
        heap.array_data(first, budget)
            .expect("Original location unchanged")
            .value,
        original
    );
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Every owned location roots its data")
            .objects,
        0
    );
    for value in values {
        heap.release_array_value(value, budget)
            .expect("Release one location");
    }
    assert_eq!(
        heap.reclaim(&[], budget).expect("No owners remain").objects,
        2
    );
    assert_eq!(
        (heap.usage().live_arrays, heap.usage().live_array_values),
        (0, 0)
    );
}

#[test]
fn seventy_thousand_simultaneous_array_data_nodes_and_value_locations_remain_distinct() {
    let (_, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let mut values = Vec::new();
    let mut identities = HashSet::new();
    for _ in 0..70_000 {
        let value = heap
            .new_array_value(budget)
            .expect("Wide data/metadata arenas")
            .value;
        assert!(identities.insert(heap.array_data(value, budget).expect("Distinct data").value));
        values.push(value);
    }
    assert_eq!(
        (
            heap.usage().live_objects,
            heap.usage().live_arrays,
            heap.usage().live_array_values
        ),
        (70_000, 70_000, 70_000)
    );
    assert_eq!(heap.usage().array_metadata_rows, 70_000);
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Registered roots survive")
            .objects,
        0
    );
    for value in values {
        heap.release_array_value(value, budget)
            .expect("Release every explicit root");
    }
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Iterative complete collection")
            .objects,
        70_000
    );
}
