// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests use valid object graphs and deliberate lifetime failures."
)]
use crate::object_heap_support::{self, allocate, number};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, StringHeap,
    StringHeapBudget,
};

#[test]
fn object_copies_share_fields_and_preserve_stored_undefined_owned_binary_keys_and_privacy() {
    let (strings, mut objects) = object_heap_support::stores(ObjectHeapBudget::default());
    let id = allocate(&mut objects);
    let alias = id;
    let mut key = b"PrivateKey\0\xff".to_vec();
    let original = key.clone();
    assert!(
        objects
            .put(
                id,
                &key,
                KernelValue::Undefined,
                &strings,
                HeapWorkBudget::default()
            )
            .expect("Store Undefined")
            .value
            .is_none()
    );
    key.fill(0);
    assert_eq!(
        objects
            .get(alias, &original, &strings, HeapWorkBudget::default())
            .expect("Owned key and alias")
            .value,
        Some(KernelValue::Undefined)
    );
    assert_eq!(objects.field_count(id), Ok(1));
    assert_eq!(
        objects
            .put(
                alias,
                &original,
                number(7),
                &strings,
                HeapWorkBudget::default()
            )
            .expect("Shared replacement")
            .value,
        Some(KernelValue::Undefined)
    );
    assert_eq!(
        objects
            .get(id, &original, &strings, HeapWorkBudget::default())
            .expect("Shared identity")
            .value,
        Some(number(7))
    );
    assert_eq!(
        objects
            .remove(id, &original, &strings, HeapWorkBudget::default())
            .expect("Remove")
            .value,
        Some(number(7))
    );
    assert!(
        objects
            .get(alias, &original, &strings, HeapWorkBudget::default())
            .expect("Missing")
            .value
            .is_none()
    );
    assert_eq!(objects.field_count(id), Ok(0));
    assert_eq!(objects.usage().index_buckets, 0);
    assert!(!format!("{id:?} {objects:?} {:?}", KernelValue::Object(id)).contains("Private"));
}

#[test]
fn stale_foreign_recreated_objects_and_wrong_or_stale_string_stores_reject_atomically() {
    let (mut strings, mut objects) = object_heap_support::stores(ObjectHeapBudget::default());
    let id = allocate(&mut objects);
    let mut other =
        ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Other namespace");
    let foreign = allocate(&mut other);
    let before = objects.usage();
    for (parent, value) in [(foreign, number(1)), (id, KernelValue::Object(foreign))] {
        assert_eq!(
            objects
                .put(
                    parent,
                    b"Private",
                    value,
                    &strings,
                    HeapWorkBudget::default()
                )
                .expect_err("Foreign object")
                .kind,
            ObjectHeapErrorKind::ForeignReference
        );
        assert_eq!(objects.usage(), before);
    }
    let wrong = StringHeap::new(StringHeapBudget::default()).expect("Other strings");
    assert_eq!(
        objects
            .get(id, b"Private", &wrong, HeapWorkBudget::default())
            .expect_err("Bound string namespace")
            .kind,
        ObjectHeapErrorKind::ForeignStringHeap
    );
    let string = strings
        .allocate(b"PrivatePayload", HeapWorkBudget::default())
        .expect("String")
        .id;
    strings
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Unrooted string");
    assert_eq!(
        objects
            .put(
                id,
                b"value",
                KernelValue::String(string),
                &strings,
                HeapWorkBudget::default()
            )
            .expect_err("Stale string")
            .kind,
        ObjectHeapErrorKind::StringHeap(HeapErrorKind::StaleReference)
    );
    assert_eq!(objects.usage(), before);
    objects
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Unrooted object");
    assert_eq!(
        objects.field_count(id),
        Err(ObjectHeapErrorKind::StaleReference)
    );
    let replacement = allocate(&mut objects);
    assert_ne!(id, replacement);
    assert_eq!(objects.usage().object_slots, 1);
    drop(objects);
    let recreated =
        ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Recreated owner");
    assert_eq!(
        recreated.field_count(replacement),
        Err(ObjectHeapErrorKind::ForeignReference)
    );
}
