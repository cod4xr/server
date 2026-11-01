// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests copy beyond native field/count widths and keep unique selection constant-work."
)]
use crate::array_copy_support::graph;
use crate::object_heap_support::{number, stores};
use cod4x_script::heap::{ArrayCowOutcome, ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn seventy_thousand_entries_copy_without_truncation_and_unique_selection_has_constant_work() {
    let mut state = graph(70_000, 0, ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let copied = state
        .heap
        .make_array_unique(state.selected, &state.strings, budget)
        .expect("Wide copy");
    assert_eq!(copied.value, ArrayCowOutcome::Detached);
    let data = state
        .heap
        .array_data(state.selected, budget)
        .expect("Copy data")
        .value;
    assert_ne!(data, state.data);
    assert_eq!(
        state
            .heap
            .array_len(data, budget)
            .expect("Wide copied count")
            .value,
        70_000
    );
    assert_eq!(state.heap.usage().live_fields, 140_000);
    let keys = state
        .heap
        .array_keys(data, budget)
        .expect("Reversed wide order");
    assert!(matches!(keys.value.first(), Some(ArrayKey::Index(0))));
    assert!(matches!(keys.value.last(), Some(ArrayKey::Index(69_999))));
    for index in 0..70_000 {
        assert_eq!(
            state
                .heap
                .array_get(data, ArrayKey::Index(index), &state.strings, budget)
                .expect("Wide copied value")
                .value,
            Some(number(i32::try_from(index).expect("Fixture value")))
        );
    }
    let before = state.heap.usage();
    let unique = state
        .heap
        .make_array_unique(
            state.selected,
            &state.strings,
            HeapWorkBudget {
                max_steps: 3,
                max_scratch_bytes: 0,
            },
        )
        .expect("Unique selection is independent of entry count");
    assert_eq!(unique.value, ArrayCowOutcome::Unchanged);
    assert_eq!(unique.usage.steps, 3);
    assert_eq!(state.heap.usage(), before);
    let (strings, mut empty) = stores(ObjectHeapBudget::default());
    let value = empty.new_array_value(budget).expect("Empty unique").value;
    assert_eq!(
        empty
            .make_array_unique(value, &strings, budget)
            .expect("Empty unique cost")
            .usage,
        unique.usage
    );
    state
        .heap
        .release_array_value(state.original, budget)
        .expect("Release source");
    assert_eq!(
        state
            .heap
            .reclaim(&[], budget)
            .expect("Collect only source")
            .fields,
        70_000
    );
}

#[test]
fn populated_selection_with_seventy_thousand_owners_changes_one_location_without_scanning_them() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    heap.array_put_unique(
        value,
        ArrayKey::Index(u64::MAX),
        number(7),
        &strings,
        budget,
    )
    .expect("Populate");
    let old = heap.array_data(value, budget).expect("Original data").value;
    let aliases: Vec<_> = (1..70_000)
        .map(|_| {
            heap.copy_array_value(value, budget)
                .expect("Wide owners")
                .value
        })
        .collect();
    let copied = heap
        .make_array_unique(
            value,
            &strings,
            HeapWorkBudget {
                max_steps: 1024,
                max_scratch_bytes: 4096,
            },
        )
        .expect("Copy one entry without traversing the owner arena");
    assert_eq!(copied.value, ArrayCowOutcome::Detached);
    assert_eq!(
        heap.array_owner_count(old, budget)
            .expect("Wide original count")
            .value,
        69_999
    );
    for alias in aliases {
        assert_eq!(
            heap.array_data(alias, budget)
                .expect("Other location unchanged")
                .value,
            old
        );
    }
    assert_ne!(
        heap.array_data(value, budget)
            .expect("Selected data changed")
            .value,
        old
    );
    assert_eq!(heap.usage().live_array_values, 70_000);
}
