// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare explicit managed operations with an independent integer-label model."
)]
use crate::object_heap_support::stores;
use cod4x_script::heap::{ArrayCowOutcome, ArrayId, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn mixed_owned_location_operations_match_an_independent_label_model() {
    let (_, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let mut random = 0x40a9_284f_b125_61e3_u64;
    let mut values = Vec::new();
    let mut labels: Vec<Option<ArrayId>> = Vec::new();
    for step in 0..16_384 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let choice = (random >> 32) % 6;
        let selected =
            usize::try_from(random >> 40).expect("Small selection") % values.len().max(1);
        if values.is_empty() || (choice == 0 && values.len() < 64) {
            let value = heap.new_array_value(budget).expect("Create").value;
            let data = heap.array_data(value, budget).expect("Data").value;
            labels.push(Some(data));
            values.push((value, labels.len() - 1));
        } else {
            let (value, label) = values[selected];
            match choice {
                1 if values.len() < 64 => {
                    let copied = heap.copy_array_value(value, budget).expect("Copy").value;
                    values.push((copied, label));
                }
                2 if values.len() < 64 => {
                    let data = labels[label].expect("Live snapshot");
                    let retained = heap.retain_array(data, budget).expect("Retain").value;
                    values.push((retained, label));
                }
                3 => {
                    heap.release_array_value(value, budget).expect("Release");
                    values.swap_remove(selected);
                }
                4 => {
                    let count = values
                        .iter()
                        .filter(|(_, current)| *current == label)
                        .count();
                    let result = heap
                        .make_empty_array_unique(value, budget)
                        .expect("Select writable value");
                    if count == 1 {
                        assert_eq!(result.value, ArrayCowOutcome::Unchanged);
                    } else {
                        assert_eq!(result.value, ArrayCowOutcome::Detached);
                        let data = heap.array_data(value, budget).expect("Detached data").value;
                        assert!(!labels.contains(&Some(data)));
                        labels.push(Some(data));
                        values[selected].1 = labels.len() - 1;
                    }
                }
                _ => collect(&mut heap, &values, &mut labels),
            }
        }
        for (label, data) in labels
            .iter()
            .enumerate()
            .filter_map(|(label, data)| data.map(|data| (label, data)))
        {
            let count = values
                .iter()
                .filter(|(_, current)| *current == label)
                .count();
            assert_eq!(
                heap.array_owner_count(data, budget)
                    .expect("Modeled count")
                    .value,
                count as u64,
                "step {step}"
            );
        }
        for (value, label) in &values {
            assert_eq!(
                heap.array_data(*value, budget)
                    .expect("Modeled identity")
                    .value,
                labels[*label].expect("Live label")
            );
        }
        assert_eq!(heap.usage().live_array_values, values.len() as u64);
    }
    for (value, _) in values.drain(..) {
        heap.release_array_value(value, budget)
            .expect("Final release");
    }
    collect(&mut heap, &values, &mut labels);
    assert_eq!(heap.usage().live_objects, 0);
}

fn collect(
    heap: &mut cod4x_script::heap::ObjectHeap,
    values: &[(cod4x_script::heap::ArrayValueId, usize)],
    labels: &mut [Option<ArrayId>],
) {
    let mut garbage = Vec::new();
    for (label, data) in labels.iter_mut().enumerate() {
        if data.is_some() && !values.iter().any(|(_, current)| *current == label) {
            garbage.push(data.take().expect("Garbage label"));
        }
    }
    let budget = HeapWorkBudget::default();
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Collect model garbage")
            .objects,
        garbage.len() as u64
    );
    for stale in garbage {
        assert!(heap.retain_array(stale, budget).is_err());
    }
}
