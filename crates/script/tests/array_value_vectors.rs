// SPDX-License-Identifier: AGPL-3.0-or-later
//! Empty-data COW state only. Host-owned aliases model the observed owner counts;
//! this does not execute retained cycle topology or implement logical alias markers.
#![allow(
    clippy::expect_used,
    reason = "Tests replay the four zero-key native alias rows."
)]
use crate::object_heap_support::stores;
use cod4x_script::heap::{ArrayCowOutcome, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn four_native_empty_data_detach_states_match_explicit_owned_aliases() -> Result<(), String> {
    let mut count = 0;
    for row in include_str!("fixtures/array_alias_vectors.txt")
        .lines()
        .filter(|row| row.starts_with("L "))
    {
        let (left, right) = row.split_once('|').ok_or("Native separator")?;
        let columns: Vec<_> = left.split_whitespace().collect();
        if columns[2] != "0" {
            continue;
        }
        let holders = columns[1]
            .parse::<u64>()
            .map_err(|error| error.to_string())?;
        let old_excess = columns[3]
            .parse::<u64>()
            .map_err(|error| error.to_string())?;
        let new_excess = columns[4]
            .parse::<u64>()
            .map_err(|error| error.to_string())?;
        assert_eq!(columns.len(), 5);
        assert!(right.trim().is_empty());
        let (_, mut heap) = stores(ObjectHeapBudget::default());
        let budget = HeapWorkBudget::default();
        let value = heap.new_array_value(budget).expect("Original value").value;
        let old_data = heap.array_data(value, budget).expect("Original data").value;
        let aliases: Vec<_> = (0..holders)
            .map(|_| {
                heap.copy_array_value(value, budget)
                    .expect("Modeled retained owner")
                    .value
            })
            .collect();
        assert_eq!(
            heap.make_empty_array_unique(value, budget)
                .expect("First write detaches")
                .value,
            ArrayCowOutcome::Detached
        );
        let new_data = heap.array_data(value, budget).expect("Copied data").value;
        assert_ne!(old_data, new_data);
        assert_eq!(
            heap.array_owner_count(old_data, budget)
                .expect("Old owners")
                .value
                - 1,
            old_excess
        );
        assert_eq!(
            heap.array_owner_count(new_data, budget)
                .expect("New owners")
                .value
                - 1,
            new_excess
        );
        assert_eq!(
            heap.make_empty_array_unique(value, budget)
                .expect("Second write unique")
                .value,
            ArrayCowOutcome::Unchanged
        );
        for alias in aliases {
            assert_eq!(
                heap.array_data(alias, budget)
                    .expect("Other aliases unchanged")
                    .value,
                old_data
            );
        }
        count += 1;
    }
    assert_eq!(count, 4);
    Ok(())
}
