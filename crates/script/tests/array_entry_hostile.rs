// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests reject stale/foreign stores and payload references before indexed publication."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn indexed_reads_and_writes_reject_foreign_stores_data_and_stale_payloads_without_mutation() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let (mut foreign_strings, mut other) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Local value").value;
    let data = heap.array_data(value, budget).expect("Local data").value;
    let foreign = other.new_array_value(budget).expect("Foreign value").value;
    let foreign_data = other
        .array_data(foreign, budget)
        .expect("Foreign data")
        .value;
    let foreign_string = foreign_strings
        .allocate(b"PrivateForeignPayload", budget)
        .expect("Foreign string")
        .id;
    let stale_object = allocate(&mut heap);
    heap.reclaim(&[], budget)
        .expect("Collect unrooted struct, retaining array");
    heap.array_put_unique(
        value,
        ArrayKey::Bytes(b"PrivateKey"),
        number(7),
        &strings,
        budget,
    )
    .expect("Existing value");
    let before = heap.usage();
    assert_eq!(
        heap.array_get(data, ArrayKey::Index(0), &foreign_strings, budget)
            .expect_err("Wrong bound string store")
            .kind,
        ObjectHeapErrorKind::ForeignStringHeap
    );
    assert_eq!(
        heap.array_get(foreign_data, ArrayKey::Index(0), &strings, budget)
            .expect_err("Wrong data namespace")
            .kind,
        ObjectHeapErrorKind::ForeignReference
    );
    assert_eq!(
        heap.array_keys(foreign_data, budget)
            .expect_err("Wrong enumeration namespace")
            .kind,
        ObjectHeapErrorKind::ForeignReference
    );
    for payload in [
        KernelValue::Object(stale_object),
        KernelValue::String(foreign_string),
        KernelValue::LocalizedString(foreign_string),
    ] {
        let error = heap
            .array_put_unique(
                value,
                ArrayKey::Bytes(b"PrivateKey"),
                payload,
                &strings,
                budget,
            )
            .expect_err("Bad replacement payload");
        assert!(!format!("{error:?} {error}").contains("Private"));
        assert_eq!(heap.usage(), before);
        assert_eq!(
            heap.array_get(data, ArrayKey::Bytes(b"PrivateKey"), &strings, budget)
                .expect("Original retained")
                .value,
            Some(number(7))
        );
    }
    assert_eq!(heap.usage(), before);
}
