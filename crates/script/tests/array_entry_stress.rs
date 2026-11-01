// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise wide typed keys and bounded retained storage under repeated indexed updates."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn seventy_thousand_mixed_keys_exceed_native_index_and_size_widths_without_truncation() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    for index in 0..70_000_u64 {
        let bytes = index.to_le_bytes();
        let key = if index.is_multiple_of(2) {
            ArrayKey::Index(u64::MAX - index)
        } else {
            ArrayKey::Bytes(&bytes)
        };
        heap.array_put_unique(
            value,
            key,
            number(i32::try_from(index).expect("Fixture value")),
            &strings,
            budget,
        )
        .expect("Wide entry");
    }
    assert_eq!(
        heap.array_len(data, budget).expect("Wide count").value,
        70_000
    );
    let keys = heap.array_keys(data, budget).expect("Wide sibling output");
    assert_eq!(keys.value.len(), 70_000);
    assert!(
        matches!(keys.value.first(), Some(ArrayKey::Bytes(bytes)) if *bytes == 69_999_u64.to_le_bytes())
    );
    assert!(matches!(keys.value.last(), Some(ArrayKey::Index(u64::MAX))));
    for index in 0..70_000_u64 {
        let bytes = index.to_le_bytes();
        let key = if index.is_multiple_of(2) {
            ArrayKey::Index(u64::MAX - index)
        } else {
            ArrayKey::Bytes(&bytes)
        };
        assert_eq!(
            heap.array_get(data, key, &strings, budget)
                .expect("Independent key lookup")
                .value,
            Some(number(i32::try_from(index).expect("Fixture value")))
        );
    }
    assert_eq!(heap.usage().key_bytes, 630_000);
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Owned root retains entries")
            .fields,
        0
    );
    heap.release_array_value(value, budget).expect("Release");
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Collect wide array")
            .fields,
        70_000
    );
}

#[test]
fn one_hundred_thousand_entry_insert_replace_remove_cycles_reuse_one_field_slot() {
    let (strings, mut heap) = stores(ObjectHeapBudget {
        max_live_fields: 1,
        max_field_slots: 1,
        max_key_bytes: 9,
        max_index_buckets: 8,
        ..ObjectHeapBudget::default()
    });
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    for index in 0..100_000 {
        let key = ArrayKey::Index(u64::MAX - u64::try_from(index).expect("Nonnegative index"));
        assert_eq!(
            heap.array_put_unique(value, key, number(index), &strings, budget)
                .expect("Insert")
                .value,
            None
        );
        assert_eq!(
            heap.array_put_unique(value, key, number(7), &strings, budget)
                .expect("Replace")
                .value,
            Some(number(index))
        );
        assert_eq!(
            heap.array_remove_unique(value, key, &strings, budget)
                .expect("Remove")
                .value,
            Some(number(7))
        );
    }
    let usage = heap.usage();
    assert_eq!(
        (
            usage.live_fields,
            usage.field_slots,
            usage.key_bytes,
            usage.index_buckets
        ),
        (0, 1, 0, 0)
    );
    assert_eq!(
        heap.array_len(data, budget)
            .expect("Empty after churn")
            .value,
        0
    );
}
