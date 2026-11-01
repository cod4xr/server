// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests retain bounded arenas during repeated populated copy/publication/reclamation."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::heap::{ArrayCowOutcome, ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn one_hundred_thousand_populated_copies_reuse_two_parent_value_and_eight_field_slots() {
    let (strings, mut heap) = stores(ObjectHeapBudget {
        max_live_objects: 2,
        max_object_slots: 2,
        max_array_values: 2,
        max_array_value_slots: 2,
        max_live_fields: 8,
        max_field_slots: 8,
        max_key_bytes: 72,
        max_index_buckets: 16,
        ..ObjectHeapBudget::default()
    });
    let budget = HeapWorkBudget::default();
    let source = heap.new_array_value(budget).expect("Source").value;
    let data = heap.array_data(source, budget).expect("Source data").value;
    for index in 0..4 {
        heap.array_put_unique(source, ArrayKey::Index(index), number(7), &strings, budget)
            .expect("Populate");
    }
    for _ in 0..100_000 {
        let selected = heap.copy_array_value(source, budget).expect("Alias").value;
        assert_eq!(
            heap.make_array_unique(selected, &strings, budget)
                .expect("Copy")
                .value,
            ArrayCowOutcome::Detached
        );
        heap.array_put_unique(selected, ArrayKey::Index(0), number(9), &strings, budget)
            .expect("Mutate copied data");
        heap.release_array_value(selected, budget)
            .expect("Release copy");
        let collection = heap.reclaim(&[], budget).expect("Collect copy");
        assert_eq!((collection.objects, collection.fields), (1, 4));
    }
    let usage = heap.usage();
    assert_eq!(
        (
            usage.live_objects,
            usage.live_arrays,
            usage.live_fields,
            usage.live_array_values
        ),
        (1, 1, 4, 1)
    );
    assert_eq!(
        (
            usage.object_slots,
            usage.array_value_slots,
            usage.field_slots,
            usage.index_buckets
        ),
        (2, 2, 8, 16)
    );
    assert_eq!(
        heap.array_data(source, budget)
            .expect("Source identity")
            .value,
        data
    );
    assert_eq!(
        heap.array_get(data, ArrayKey::Index(0), &strings, budget)
            .expect("Source content")
            .value,
        Some(number(7))
    );
}
