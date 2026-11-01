// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests preserve array-contained object graphs and strings across explicit collections."
)]
use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn managed_array_roots_retain_struct_cycles_and_plain_localized_strings_until_release() {
    let (mut strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Array root").value;
    let data = heap.array_data(value, budget).expect("Snapshot").value;
    let object = allocate(&mut heap);
    let plain = strings
        .allocate(b"PrivateArrayPlain", budget)
        .expect("Plain")
        .id;
    let localized = strings
        .allocate(b"PrivateArrayLocalized", budget)
        .expect("Localized")
        .id;
    heap.put(
        object,
        b"self",
        KernelValue::Object(object),
        &strings,
        budget,
    )
    .expect("Cycle");
    for (index, element) in [
        KernelValue::Object(object),
        KernelValue::String(plain),
        KernelValue::LocalizedString(localized),
    ]
    .into_iter()
    .enumerate()
    {
        heap.array_put_unique(
            value,
            ArrayKey::Index(index as u64),
            element,
            &strings,
            budget,
        )
        .expect("Contained root");
    }
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Host-owned location is a root")
            .objects,
        0
    );
    assert_eq!(
        heap.analyze_cycles(&[], budget)
            .expect("Cycle reachable through array")
            .unreachable_objects,
        0
    );
    assert_eq!(
        heap.reclaim_strings(&mut strings, &[], budget)
            .expect("Composite fields retain both tags")
            .strings,
        0
    );
    assert_eq!(
        heap.array_get(data, ArrayKey::Index(0), &strings, budget)
            .expect("Same struct identity")
            .value,
        Some(KernelValue::Object(object))
    );
    assert!(!format!("{heap:?}").contains("PrivateArray"));
    heap.release_array_value(value, budget)
        .expect("Release root");
    let collection = heap
        .reclaim(&[], budget)
        .expect("Collect array and struct cycle");
    assert_eq!((collection.objects, collection.fields), (2, 4));
    assert_eq!(
        heap.reclaim_strings(&mut strings, &[], budget)
            .expect("Collect released nested strings")
            .strings,
        2
    );
    assert_eq!(
        heap.array_len(data, budget)
            .expect_err("Stale snapshot")
            .kind,
        ObjectHeapErrorKind::StaleReference
    );
    assert_eq!(heap.usage().key_bytes, 0);
}
