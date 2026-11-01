// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise repeated managed ownership with bounded retained arenas."
)]
use crate::object_heap_support::stores;
use cod4x_script::heap::{ArrayCowOutcome, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn one_hundred_thousand_copy_detach_release_cycles_reuse_two_data_and_value_slots() {
    let (_, mut heap) = stores(ObjectHeapBudget {
        max_live_objects: 2,
        max_object_slots: 2,
        max_array_values: 2,
        max_array_value_slots: 2,
        ..ObjectHeapBudget::default()
    });
    let budget = HeapWorkBudget::default();
    let original = heap
        .new_array_value(budget)
        .expect("Persistent value")
        .value;
    let data = heap
        .array_data(original, budget)
        .expect("Persistent data")
        .value;
    let mut work = 0;
    for _ in 0..100_000 {
        let copied = heap.copy_array_value(original, budget).expect("Copy");
        work += copied.usage.steps;
        let detached = heap
            .make_empty_array_unique(copied.value, budget)
            .expect("Detach");
        assert_eq!(detached.value, ArrayCowOutcome::Detached);
        work += detached.usage.steps;
        work += heap
            .release_array_value(copied.value, budget)
            .expect("Release")
            .usage
            .steps;
        let collection = heap.reclaim(&[], budget).expect("Collect detached data");
        assert_eq!(collection.objects, 1);
        work += collection.usage.steps;
    }
    let usage = heap.usage();
    assert_eq!(
        (
            usage.live_objects,
            usage.live_arrays,
            usage.live_array_values
        ),
        (1, 1, 1)
    );
    assert_eq!(
        (
            usage.object_slots,
            usage.array_value_slots,
            usage.array_metadata_rows
        ),
        (2, 2, 2)
    );
    assert_eq!(
        heap.array_data(original, budget)
            .expect("Same persistent data")
            .value,
        data
    );
    assert_eq!(
        heap.array_owner_count(data, budget)
            .expect("One owner")
            .value,
        1
    );
    assert!(work > 100_000);
}
