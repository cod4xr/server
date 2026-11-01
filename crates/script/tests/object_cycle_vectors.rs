// SPDX-License-Identifier: AGPL-3.0-or-later
//! Topology replay only: array payloads are represented by ordinary graph nodes.
//! This does not implement or execute Rust array COW.
#![allow(
    clippy::expect_used,
    reason = "Tests match native cycle-alias topology and classify old versus copied live targets."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn native_cycle_alias_traces_match_graph_boundary_targets_without_inheriting_them_on_fresh_copies()
-> Result<(), String> {
    let mut count = 0;
    for row in include_str!("fixtures/array_alias_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let (original, copied) = row.split_once('|').ok_or("Native key separator")?;
        let columns: Vec<_> = original.split_whitespace().collect();
        assert!(columns.len() >= 5 && columns[0] == "L");
        let holders: u64 = columns[1]
            .parse()
            .map_err(|error: std::num::ParseIntError| error.to_string())?;
        let keys: i32 = columns[2]
            .parse()
            .map_err(|error: std::num::ParseIntError| error.to_string())?;
        assert_eq!(
            columns[3]
                .parse::<u64>()
                .map_err(|error| error.to_string())?,
            holders - 1
        );
        assert_eq!(columns[4], "0");
        let old_keys = key_values(columns[5..].iter().copied())?;
        let new_keys = key_values(copied.split_whitespace())?;
        assert_eq!(old_keys, (0..keys).rev().collect::<Vec<_>>());
        assert_eq!(new_keys, (0..keys).collect::<Vec<_>>());
        let (strings, mut heap) = stores(ObjectHeapBudget::default());
        let original = allocate(&mut heap);
        for key in 0..keys {
            heap.put(
                original,
                format!("i{key}").as_bytes(),
                number(key),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Modeled integer payload");
        }
        for _ in 0..holders {
            let cycle = allocate(&mut heap);
            heap.put(
                cycle,
                b"self",
                KernelValue::Object(cycle),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Self cycle");
            heap.put(
                cycle,
                b"alias",
                KernelValue::Object(original),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Retained outgoing alias");
        }
        let report = heap
            .analyze_cycles(&[original], HeapWorkBudget::default())
            .expect("Original boundary");
        assert_eq!(
            (
                report.unreachable_objects,
                report.cycle_retained_objects,
                report.cycle_to_live_edges
            ),
            (holders, holders, holders)
        );
        assert_eq!(report.live_targets(), &[original]);
        let copied = allocate(&mut heap);
        for key in 0..keys {
            heap.put(
                copied,
                format!("i{key}").as_bytes(),
                number(key),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Fresh copied payload model");
        }
        let report = heap
            .analyze_cycles(&[copied], HeapWorkBudget::default())
            .expect("Fresh target has no retained incoming alias");
        assert_eq!(
            (
                report.unreachable_objects,
                report.cycle_retained_objects,
                report.cycle_to_live_edges
            ),
            (holders + 1, holders + 1, 0)
        );
        assert!(report.live_targets().is_empty());
        count += 1;
    }
    assert_eq!(count, 20);
    Ok(())
}

fn key_values<'a>(keys: impl Iterator<Item = &'a str>) -> Result<Vec<i32>, String> {
    keys.map(|key| {
        key.strip_prefix("i:")
            .ok_or_else(|| "Unexpected native key type".to_owned())?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    })
    .collect()
}
