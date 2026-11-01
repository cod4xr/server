// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native 20-state COW replay with host aliases, not retained-cycle/marker behavior.
#![allow(
    clippy::expect_used,
    reason = "Tests reproduce scalar native copy states using explicitly owned host aliases."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::heap::{ArrayCowOutcome, ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn twenty_native_alias_copy_states_preserve_owner_counts_and_key_reversal() -> Result<(), String> {
    let mut count = 0;
    for row in include_str!("fixtures/array_alias_vectors.txt")
        .lines()
        .filter(|row| row.starts_with("L "))
    {
        let (left, right) = row.split_once('|').ok_or("Native separator")?;
        let columns: Vec<_> = left.split_whitespace().collect();
        let holders = columns[1]
            .parse::<u64>()
            .map_err(|error| error.to_string())?;
        let width = columns[2]
            .parse::<u64>()
            .map_err(|error| error.to_string())?;
        let old_excess = columns[3]
            .parse::<u64>()
            .map_err(|error| error.to_string())?;
        let new_excess = columns[4]
            .parse::<u64>()
            .map_err(|error| error.to_string())?;
        let (strings, mut heap) = stores(ObjectHeapBudget::default());
        let budget = HeapWorkBudget::default();
        let value = heap.new_array_value(budget).expect("Original").value;
        let old = heap.array_data(value, budget).expect("Original data").value;
        for index in 0..width {
            heap.array_put_unique(value, ArrayKey::Index(index), number(7), &strings, budget)
                .expect("Populate");
        }
        let aliases: Vec<_> = (0..holders)
            .map(|_| {
                heap.copy_array_value(value, budget)
                    .expect("Modeled retained alias")
                    .value
            })
            .collect();
        assert_eq!(
            heap.make_array_unique(value, &strings, budget)
                .expect("Copy supported entries")
                .value,
            ArrayCowOutcome::Detached
        );
        let copied = heap.array_data(value, budget).expect("Copy data").value;
        assert_eq!(
            heap.array_owner_count(old, budget)
                .expect("Old owners")
                .value
                - 1,
            old_excess
        );
        assert_eq!(
            heap.array_owner_count(copied, budget)
                .expect("New owners")
                .value
                - 1,
            new_excess
        );
        assert_eq!(
            key_words(heap.array_keys(old, budget).expect("Old keys").value)?,
            columns[5..]
        );
        assert_eq!(
            key_words(heap.array_keys(copied, budget).expect("Copied keys").value)?,
            right.split_whitespace().collect::<Vec<_>>()
        );
        assert_eq!(
            heap.make_array_unique(value, &strings, budget)
                .expect("Second selection")
                .value,
            ArrayCowOutcome::Unchanged
        );
        for alias in aliases {
            assert_eq!(
                heap.array_data(alias, budget).expect("Other owner").value,
                old
            );
        }
        count += 1;
    }
    assert_eq!(count, 20);
    Ok(())
}

fn key_words(keys: Vec<ArrayKey<'_>>) -> Result<Vec<String>, String> {
    keys.into_iter()
        .map(|key| match key {
            ArrayKey::Index(index) => Ok(format!("i:{index}")),
            ArrayKey::Bytes(_) => Err("Expected native integer key".into()),
        })
        .collect()
}
