// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers build deterministic shared arrays and spare field/index states."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::heap::{
    ArrayId, ArrayKey, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapUsage,
    StringHeap,
};

pub mod model;
mod model_step;

pub struct Graph {
    pub strings: StringHeap,
    pub heap: ObjectHeap,
    pub original: ArrayValueId,
    pub selected: ArrayValueId,
    pub data: ArrayId,
    pub width: u64,
}

pub fn graph(width: u64, spare: u64, limits: ObjectHeapBudget) -> Graph {
    let (strings, mut heap) = stores(limits);
    let budget = HeapWorkBudget::default();
    let original = heap.new_array_value(budget).expect("Original").value;
    for index in 0..width {
        heap.array_put_unique(
            original,
            ArrayKey::Index(index),
            number(i32::try_from(index).expect("Fixture value")),
            &strings,
            budget,
        )
        .expect("Source entry");
    }
    if spare > 0 {
        let other = allocate(&mut heap);
        for index in 0..spare {
            heap.put(other, &index.to_le_bytes(), number(0), &strings, budget)
                .expect("Spare field");
        }
        for index in 0..spare {
            heap.remove(other, &index.to_le_bytes(), &strings, budget)
                .expect("Reusable field/tombstone");
        }
    }
    let selected = heap
        .copy_array_value(original, budget)
        .expect("Shared selected location")
        .value;
    let data = heap.array_data(original, budget).expect("Data").value;
    Graph {
        strings,
        heap,
        original,
        selected,
        data,
        width,
    }
}

pub fn unchanged(graph: &Graph, before: ObjectHeapUsage) {
    let budget = HeapWorkBudget::default();
    assert_eq!(graph.heap.usage(), before);
    assert_eq!(
        graph
            .heap
            .array_data(graph.selected, budget)
            .expect("Selected identity unchanged")
            .value,
        graph.data
    );
    assert_eq!(
        graph
            .heap
            .array_data(graph.original, budget)
            .expect("Original identity unchanged")
            .value,
        graph.data
    );
    assert_eq!(
        graph
            .heap
            .array_owner_count(graph.data, budget)
            .expect("Owner count unchanged")
            .value,
        2
    );
    assert_eq!(
        graph
            .heap
            .array_len(graph.data, budget)
            .expect("Length unchanged")
            .value,
        graph.width
    );
    for index in 0..graph.width {
        assert_eq!(
            graph
                .heap
                .array_get(graph.data, ArrayKey::Index(index), &graph.strings, budget)
                .expect("Unchanged content")
                .value,
            Some(number(i32::try_from(index).expect("Fixture value")))
        );
    }
}
