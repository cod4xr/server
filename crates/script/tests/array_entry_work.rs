// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust every failed work prefix of indexed operations and key-list scratch."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::heap::ArrayKey;
use cod4x_script::heap::{
    ArrayValueId, HeapWorkBudget, HeapWorkUsage, ObjectHeap, ObjectHeapBudget, ObjectHeapError,
    StringHeap,
};

fn graph() -> (StringHeap, ObjectHeap, ArrayValueId) {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    for index in 0..4 {
        heap.array_put_unique(value, ArrayKey::Index(index), number(8), &strings, budget)
            .expect("Fill initial bucket boundary");
    }
    (strings, heap, value)
}

fn operate(
    heap: &mut ObjectHeap,
    strings: &StringHeap,
    value: ArrayValueId,
    operation: usize,
    budget: HeapWorkBudget,
) -> Result<HeapWorkUsage, ObjectHeapError> {
    let data = heap.array_data(value, HeapWorkBudget::default())?.value;
    match operation {
        0 => heap
            .array_put_unique(
                value,
                ArrayKey::Bytes(&[42; 120]),
                number(9),
                strings,
                budget,
            )
            .map(|result| result.usage),
        1 => heap
            .array_put_unique(value, ArrayKey::Index(0), number(9), strings, budget)
            .map(|result| result.usage),
        2 => heap
            .array_remove_unique(value, ArrayKey::Index(2), strings, budget)
            .map(|result| result.usage),
        3 => heap
            .array_get(data, ArrayKey::Index(3), strings, budget)
            .map(|result| result.usage),
        _ => heap.array_keys(data, budget).map(|result| result.usage),
    }
}

#[test]
fn entry_operations_preflight_every_work_prefix_and_exact_key_list_scratch() {
    for operation in 0..5 {
        let (strings, mut heap, value) = graph();
        let usage = operate(
            &mut heap,
            &strings,
            value,
            operation,
            HeapWorkBudget::default(),
        )
        .expect("Measure operation");
        for maximum in 0..=usage.steps {
            let (strings, mut heap, value) = graph();
            let data = heap
                .array_data(value, HeapWorkBudget::default())
                .expect("Data")
                .value;
            let before = heap.usage();
            let result = operate(
                &mut heap,
                &strings,
                value,
                operation,
                HeapWorkBudget {
                    max_steps: maximum,
                    max_scratch_bytes: usage.scratch_bytes,
                },
            );
            if maximum < usage.steps {
                let error = result.expect_err("Every incomplete prefix");
                assert!(error.usage.steps <= maximum);
                assert_eq!(heap.usage(), before);
                assert_eq!(
                    heap.array_len(data, HeapWorkBudget::default())
                        .expect("No partial insert/delete")
                        .value,
                    4
                );
                for index in 0..4 {
                    assert_eq!(
                        heap.array_get(
                            data,
                            ArrayKey::Index(index),
                            &strings,
                            HeapWorkBudget::default()
                        )
                        .expect("Unchanged entry")
                        .value,
                        Some(number(8))
                    );
                }
            } else {
                assert_eq!(result.expect("Exact work and scratch"), usage);
            }
        }
        if usage.scratch_bytes > 0 {
            let (strings, mut heap, value) = graph();
            let before = heap.usage();
            assert!(
                operate(
                    &mut heap,
                    &strings,
                    value,
                    operation,
                    HeapWorkBudget {
                        max_scratch_bytes: usage.scratch_bytes - 1,
                        ..HeapWorkBudget::default()
                    }
                )
                .is_err()
            );
            assert_eq!(heap.usage(), before);
        }
    }
}
