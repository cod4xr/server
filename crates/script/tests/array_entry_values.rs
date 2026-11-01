// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests check exact typed keys, primitive payloads and shared-write rejection."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn wide_integer_and_binary_keys_remain_distinct_owned_and_ordered_without_payload_conversion() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    let mut private_key = vec![0, 42, 0, 255];
    for (key, element) in [
        (ArrayKey::Index(u64::MAX), number(1)),
        (ArrayKey::Bytes(&private_key), number(2)),
        (ArrayKey::Bytes(b""), KernelValue::Undefined),
        (ArrayKey::Index(0), number(4)),
    ] {
        assert_eq!(
            heap.array_put_unique(value, key, element, &strings, budget)
                .expect("Insert")
                .value,
            None
        );
    }
    private_key.fill(7);
    assert_eq!(
        heap.array_len(data, budget)
            .expect("Four distinct keys")
            .value,
        4
    );
    assert_eq!(
        heap.array_get(data, ArrayKey::Bytes(&[0, 42, 0, 255]), &strings, budget)
            .expect("Owned binary key")
            .value,
        Some(number(2))
    );
    assert_eq!(
        heap.array_get(data, ArrayKey::Bytes(b""), &strings, budget)
            .expect("Stored Undefined is present")
            .value,
        Some(KernelValue::Undefined)
    );
    assert_eq!(
        heap.array_get(data, ArrayKey::Bytes(b"missing"), &strings, budget)
            .expect("Absent")
            .value,
        None
    );
    assert_eq!(
        heap.array_put_unique(
            value,
            ArrayKey::Index(u64::MAX),
            number(8),
            &strings,
            budget
        )
        .expect("Replace without reordering")
        .value,
        Some(number(1))
    );
    let keys = heap.array_keys(data, budget).expect("Sibling order");
    assert!(matches!(
        keys.value.as_slice(),
        [
            ArrayKey::Index(0),
            ArrayKey::Bytes(b""),
            ArrayKey::Bytes([0, 42, 0, 255]),
            ArrayKey::Index(u64::MAX)
        ]
    ));
    assert_eq!(
        keys.usage.scratch_bytes,
        4 * size_of::<ArrayKey<'_>>() as u64
    );
    assert_eq!(heap.usage().key_bytes, 9 + 5 + 1 + 9);
    assert!(!format!("{keys:?} {heap:?}").contains("missing"));
    heap.array_remove_unique(value, ArrayKey::Bytes(b""), &strings, budget)
        .expect("Remove stored Undefined");
    assert_eq!(
        heap.array_get(data, ArrayKey::Bytes(b""), &strings, budget)
            .expect("Now absent")
            .value,
        None
    );
    assert_eq!(heap.array_len(data, budget).expect("Three remain").value, 3);
}

#[test]
fn shared_entry_mutation_and_empty_only_copy_reject_without_identity_or_field_changes() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    heap.array_put_unique(value, ArrayKey::Index(7), number(5), &strings, budget)
        .expect("Populate unique value");
    let alias = heap.copy_array_value(value, budget).expect("Alias").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    let before = heap.usage();
    assert_eq!(
        heap.array_put_unique(alias, ArrayKey::Index(7), number(9), &strings, budget)
            .expect_err("No raw shared write")
            .kind,
        ObjectHeapErrorKind::SharedArray
    );
    assert_eq!(
        heap.array_remove_unique(alias, ArrayKey::Index(7), &strings, budget)
            .expect_err("No raw shared delete")
            .kind,
        ObjectHeapErrorKind::SharedArray
    );
    assert_eq!(
        heap.make_empty_array_unique(alias, budget)
            .expect_err("Empty-only copy cannot discard content")
            .kind,
        ObjectHeapErrorKind::ExpectedEmptyArray
    );
    assert_eq!(heap.usage(), before);
    assert_eq!(
        heap.array_data(alias, budget)
            .expect("Unchanged identity")
            .value,
        data
    );
    assert_eq!(
        heap.array_get(data, ArrayKey::Index(7), &strings, budget)
            .expect("Unchanged field")
            .value,
        Some(number(5))
    );
    heap.release_array_value(value, budget)
        .expect("Release other owner");
    heap.array_put_unique(alias, ArrayKey::Index(7), number(9), &strings, budget)
        .expect("Now unique");
}
