// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests distinguish explicitly owned array values, data snapshots and raw handle copies."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::heap::{ArrayCowOutcome, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn explicit_value_copies_detach_empty_data_and_remain_roots_until_release() {
    let (_strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let first = heap
        .new_array_value(budget)
        .expect("Owned empty value")
        .value;
    let raw_copy = first;
    let data = heap
        .array_data(first, budget)
        .expect("Nonowning snapshot")
        .value;
    assert_eq!(
        heap.array_owner_count(data, budget)
            .expect("Raw copies are not owners")
            .value,
        1
    );
    assert_eq!(heap.array_len(data, budget).expect("Empty data").value, 0);
    let alias = heap
        .copy_array_value(first, budget)
        .expect("Second owned location")
        .value;
    assert_ne!(first, alias);
    assert_eq!(
        heap.array_data(alias, budget).expect("Shared data").value,
        data
    );
    assert_eq!(
        heap.array_owner_count(data, budget)
            .expect("Two locations")
            .value,
        2
    );
    heap.release_array_value(first, budget)
        .expect("Release exact location");
    assert_eq!(
        heap.array_data(raw_copy, budget)
            .expect_err("Raw copied handles become stale together")
            .kind,
        ObjectHeapErrorKind::StaleArrayValue
    );
    let retained = heap
        .retain_array(data, budget)
        .expect("Retain still-live snapshot")
        .value;
    assert_ne!(retained, first);
    assert_eq!(heap.usage().array_value_slots, 2);
    assert_eq!(
        heap.make_empty_array_unique(alias, budget)
            .expect("Detach selected alias")
            .value,
        ArrayCowOutcome::Detached
    );
    let detached = heap
        .array_data(alias, budget)
        .expect("Fresh data identity")
        .value;
    assert_ne!(detached, data);
    assert_eq!(
        heap.array_data(retained, budget)
            .expect("Other location unchanged")
            .value,
        data
    );
    assert_eq!(
        heap.make_empty_array_unique(alias, budget)
            .expect("Second write stays unique")
            .value,
        ArrayCowOutcome::Unchanged
    );
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Owned locations are explicit roots")
            .objects,
        0
    );
    assert_eq!(
        heap.analyze_cycles(&[], budget)
            .expect("Analysis includes managed roots")
            .unreachable_objects,
        0
    );
    assert_eq!(format!("{first:?}"), "ArrayValueId(..)");
    assert_eq!(format!("{data:?}"), "ArrayId(..)");
}

#[test]
fn data_and_value_foreign_namespaces_and_released_generations_reject_without_mutation() {
    let (_, mut heap) = stores(ObjectHeapBudget::default());
    let (_, mut other) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let foreign = other.new_array_value(budget).expect("Foreign value").value;
    let foreign_data = other
        .array_data(foreign, budget)
        .expect("Foreign data")
        .value;
    let value = heap.new_array_value(budget).expect("Local value").value;
    let snapshot = heap.array_data(value, budget).expect("Local data").value;
    let before = heap.usage();
    assert_eq!(
        heap.copy_array_value(foreign, budget)
            .expect_err("Value namespace")
            .kind,
        ObjectHeapErrorKind::ForeignArrayValue
    );
    assert_eq!(
        heap.retain_array(foreign_data, budget)
            .expect_err("Data namespace")
            .kind,
        ObjectHeapErrorKind::ForeignReference
    );
    assert_eq!(heap.usage(), before);
    heap.release_array_value(value, budget).expect("Release");
    let next = heap
        .retain_array(snapshot, budget)
        .expect("Reuse location generation")
        .value;
    assert_ne!(value, next);
    assert_eq!(
        heap.copy_array_value(value, budget)
            .expect_err("No stale clone")
            .kind,
        ObjectHeapErrorKind::StaleArrayValue
    );
    assert_eq!(
        heap.release_array_value(value, budget)
            .expect_err("No double release")
            .kind,
        ObjectHeapErrorKind::StaleArrayValue
    );
    assert_eq!(
        heap.array_owner_count(snapshot, budget)
            .expect("Counter remains exact")
            .value,
        1
    );
}

#[test]
fn released_unowned_data_collects_and_reused_parent_slots_lose_array_classification() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let first = heap.new_array_value(budget).expect("First data").value;
    let alias = first;
    let data = heap.array_data(first, budget).expect("Snapshot").value;
    let retained = heap.new_array_value(budget).expect("Second data").value;
    heap.release_array_value(alias, budget)
        .expect("Release copy");
    heap.release_array_value(retained, budget)
        .expect("Release old data");
    assert_eq!(heap.usage().live_array_values, 0);
    assert_eq!(
        heap.array_owner_count(data, budget)
            .expect("Snapshot remains nonowning")
            .value,
        0
    );
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Explicit data collection")
            .objects,
        2
    );
    assert_eq!(heap.usage().live_arrays, 0);
    assert_eq!(
        heap.array_len(data, budget)
            .expect_err("Snapshot is stale after collection")
            .kind,
        ObjectHeapErrorKind::StaleReference
    );
    let ordinary = allocate(&mut heap);
    heap.put(ordinary, b"ordinary", number(7), &strings, budget)
        .expect("Reused parent slot loses array classification");
    assert_eq!(heap.field_count(ordinary), Ok(1));
    assert!(!format!("{first:?} {data:?} {heap:?}").contains("ordinary"));
}
