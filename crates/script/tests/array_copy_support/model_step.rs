// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Model operations compare explicit data versions, owner counts and key-order rules."
)]
use crate::array_copy_support::model::{Model, Version};
use crate::object_heap_support::number;
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{ArrayCowOutcome, ArrayKey, HeapWorkBudget, ObjectHeap, StringHeap};
use std::collections::BTreeMap;

pub(super) fn step(model: &mut Model, heap: &mut ObjectHeap, strings: &StringHeap, random: u64) {
    let budget = HeapWorkBudget::default();
    let operation = (random >> 32) % 6;
    if model.values.is_empty() || (operation == 0 && model.values.len() < 16) {
        let value = heap.new_array_value(budget).expect("New value").value;
        let data = heap.array_data(value, budget).expect("New data").value;
        assert!(!model.versions.iter().any(|version| version.data == data));
        model.versions.push(Version {
            data,
            entries: BTreeMap::new(),
            order: Vec::new(),
            alive: true,
        });
        model.values.push((value, model.versions.len() - 1));
        return;
    }
    let selected = usize::try_from(random >> 40).expect("Small selection") % model.values.len();
    let (value, label) = model.values[selected];
    match operation {
        1 if model.values.len() < 16 => {
            let copied = heap
                .copy_array_value(value, budget)
                .expect("Copy location")
                .value;
            model.values.push((copied, label));
        }
        2 => {
            heap.release_array_value(value, budget)
                .expect("Release location");
            model.values.swap_remove(selected);
            assert!(heap.array_data(value, budget).is_err());
        }
        3 if model.values.len() < 16 => {
            let count = model
                .versions
                .iter()
                .filter(|version| version.alive)
                .count();
            let chosen = usize::try_from(random >> 48).expect("Small retained selection") % count;
            let (label, version) = model
                .versions
                .iter()
                .enumerate()
                .filter(|(_, version)| version.alive)
                .nth(chosen)
                .expect("Live snapshot");
            let retained = heap
                .retain_array(version.data, budget)
                .expect("Retain snapshot")
                .value;
            model.values.push((retained, label));
        }
        _ => mutate(model, heap, strings, selected, random),
    }
}

fn mutate(
    model: &mut Model,
    heap: &mut ObjectHeap,
    strings: &StringHeap,
    selected: usize,
    random: u64,
) {
    let budget = HeapWorkBudget::default();
    let (value, mut label) = model.values[selected];
    let owners = model
        .values
        .iter()
        .filter(|(_, current)| *current == label)
        .count();
    let result = heap
        .make_array_unique(value, strings, budget)
        .expect("Select data for mutation");
    if owners > 1 {
        assert_eq!(result.value, ArrayCowOutcome::Detached);
        let previous = &model.versions[label];
        let data = heap.array_data(value, budget).expect("Detached data").value;
        assert!(!model.versions.iter().any(|version| version.data == data));
        let mut order = previous.order.clone();
        order.reverse();
        model.versions.push(Version {
            data,
            entries: previous.entries.clone(),
            order,
            alive: true,
        });
        label = model.versions.len() - 1;
        model.values[selected].1 = label;
    } else {
        assert_eq!(result.value, ArrayCowOutcome::Unchanged);
    }
    let key_label = ((random >> 20) % 2, (1 << 48) + ((random >> 12) % 9));
    let bytes = key_label.1.to_le_bytes();
    let key = if key_label.0 == 0 {
        ArrayKey::Index(key_label.1)
    } else {
        ArrayKey::Bytes(&bytes)
    };
    let version = &mut model.versions[label];
    if random.is_multiple_of(3) {
        let old = version.entries.remove(&key_label);
        version.order.retain(|current| *current != key_label);
        assert_eq!(
            heap.array_remove_unique(value, key, strings, budget)
                .expect("Remove entry")
                .value,
            old
        );
    } else {
        let element = if random.is_multiple_of(7) {
            KernelValue::Undefined
        } else {
            number(i32::try_from(random >> 40).expect("Small payload"))
        };
        let old = version.entries.insert(key_label, element);
        if old.is_none() {
            version.order.insert(0, key_label);
        }
        assert_eq!(
            heap.array_put_unique(value, key, element, strings, budget)
                .expect("Put entry")
                .value,
            old
        );
    }
    for ((kind, index), element) in &version.entries {
        let bytes = index.to_le_bytes();
        let key = if *kind == 0 {
            ArrayKey::Index(*index)
        } else {
            ArrayKey::Bytes(&bytes)
        };
        assert_eq!(
            heap.array_get(version.data, key, strings, budget)
                .expect("Independent copied payload")
                .value,
            Some(*element)
        );
    }
}
