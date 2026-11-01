// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected U/V/F state replay, not native free-list/allocator or source assignment parity.
#![allow(
    clippy::expect_used,
    reason = "Tests replay raw setter, replacement order and delete/reinsert semantic states."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn three_native_unique_entry_traces_preserve_replacement_order_raw_undefined_and_reinsertion() {
    let fixture = include_str!("fixtures/array_vectors.txt");
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    for index in 0..3 {
        heap.array_put_unique(value, ArrayKey::Index(index), number(7), &strings, budget)
            .expect("Populate");
    }
    heap.array_put_unique(value, ArrayKey::Index(1), number(77), &strings, budget)
        .expect("Replace middle");
    let keys = heap.array_keys(data, budget).expect("Native U order");
    assert!(matches!(
        keys.value.as_slice(),
        [ArrayKey::Index(2), ArrayKey::Index(1), ArrayKey::Index(0)]
    ));
    assert!(fixture.lines().any(|row| row == "U i:2 i:1 i:0"));
    for index in [2, 0, 1] {
        heap.array_remove_unique(value, ArrayKey::Index(index), &strings, budget)
            .expect("Native removal order");
    }
    heap.array_put_unique(value, ArrayKey::Index(10), number(7), &strings, budget)
        .expect("Reinsert");
    assert!(matches!(
        heap.array_keys(data, budget)
            .expect("F key result")
            .value
            .as_slice(),
        [ArrayKey::Index(10)]
    ));
    assert!(fixture.lines().any(|row| row == "F 3 1 i:10"));
    heap.array_remove_unique(value, ArrayKey::Index(10), &strings, budget)
        .expect("Reset");
    heap.array_put_unique(value, ArrayKey::Index(7), number(42), &strings, budget)
        .expect("Raw setter initial value");
    heap.array_put_unique(
        value,
        ArrayKey::Index(7),
        KernelValue::Undefined,
        &strings,
        budget,
    )
    .expect("Raw setter preserves Undefined entry");
    assert_eq!(heap.array_len(data, budget).expect("V live entry").value, 1);
    assert_eq!(
        heap.array_get(data, ArrayKey::Index(7), &strings, budget)
            .expect("V raw Undefined")
            .value,
        Some(KernelValue::Undefined)
    );
    assert!(fixture.lines().any(|row| row == "V 0 1 i:7 /0"));
    heap.array_remove_unique(value, ArrayKey::Index(7), &strings, budget)
        .expect("Explicit removal");
    assert_eq!(
        heap.array_len(data, budget).expect("V final state").value,
        0
    );
}
