// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests build finite deep/shared/cyclic owned graphs."
)]
use crate::object_heap_support::{self, allocate, number};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn shared_and_cyclic_graphs_trace_iteratively_and_collect_only_unreachable_objects() {
    let (strings, mut objects) = object_heap_support::stores(ObjectHeapBudget::default());
    let head = allocate(&mut objects);
    let mut tail = head;
    for _ in 1..12_000 {
        let next = allocate(&mut objects);
        objects
            .put(
                tail,
                b"next",
                KernelValue::Object(next),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Chain edge");
        tail = next;
    }
    objects
        .put(
            tail,
            b"next",
            KernelValue::Object(head),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Cycle");
    let observer = allocate(&mut objects);
    objects
        .put(
            observer,
            b"shared",
            KernelValue::Object(tail),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Shared edge");
    let garbage = allocate(&mut objects);
    objects
        .put(
            garbage,
            b"self",
            KernelValue::Object(garbage),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Unreachable cycle");
    let collection = objects
        .reclaim(&[head, observer, head], HeapWorkBudget::default())
        .expect("Complete graph");
    assert_eq!((collection.objects, collection.fields), (1, 1));
    assert_eq!(
        objects.field_count(garbage),
        Err(ObjectHeapErrorKind::StaleReference)
    );
    assert_eq!(objects.usage().live_objects, 12_001);
    assert_eq!(objects.usage().live_fields, 12_001);
    assert_eq!(
        objects
            .get(tail, b"next", &strings, HeapWorkBudget::default())
            .expect("Cycle survives")
            .value,
        Some(KernelValue::Object(head))
    );
    let collection = objects
        .reclaim(&[], HeapWorkBudget::default())
        .expect("No recursive destruction");
    assert_eq!((collection.objects, collection.fields), (12_001, 12_001));
    assert_eq!(objects.usage().index_buckets, 0);
    assert_eq!(objects.usage().key_bytes, 0);
    let reused = allocate(&mut objects);
    assert_ne!(reused, head);
    objects
        .put(
            reused,
            b"fresh",
            number(42),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Reused storage");
}

#[test]
fn string_collection_preserves_plain_localized_nested_and_extra_roots_until_object_collection() {
    let (mut strings, mut objects) = object_heap_support::stores(ObjectHeapBudget::default());
    let root = allocate(&mut objects);
    let child = allocate(&mut objects);
    objects
        .put(
            root,
            b"child",
            KernelValue::Object(child),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Child");
    let plain = strings
        .allocate(b"plain", HeapWorkBudget::default())
        .expect("Plain")
        .id;
    let localized = strings
        .allocate(b"MENU", HeapWorkBudget::default())
        .expect("Localized")
        .id;
    let host = strings
        .allocate(b"host", HeapWorkBudget::default())
        .expect("Extra root")
        .id;
    let garbage = strings
        .allocate(b"garbage", HeapWorkBudget::default())
        .expect("Garbage")
        .id;
    objects
        .put(
            root,
            b"plain",
            KernelValue::String(plain),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Plain field");
    objects
        .put(
            child,
            b"localized",
            KernelValue::LocalizedString(localized),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Localized field");
    let result = objects
        .reclaim_strings(&mut strings, &[host], HeapWorkBudget::default())
        .expect("Stored and supplied roots");
    assert_eq!(result.strings, 1);
    assert!(strings.get(garbage).is_err());
    assert_eq!(strings.get(plain).expect("Plain survives"), b"plain");
    assert_eq!(strings.get(localized).expect("Localized survives"), b"MENU");
    objects
        .reclaim(&[root], HeapWorkBudget::default())
        .expect("Nested object root");
    assert_eq!(
        objects
            .reclaim_strings(&mut strings, &[host], HeapWorkBudget::default())
            .expect("All live")
            .strings,
        0
    );
    objects
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Drop graph");
    assert_eq!(
        objects
            .reclaim_strings(&mut strings, &[host], HeapWorkBudget::default())
            .expect("Only host survives")
            .strings,
        2
    );
    assert_eq!(strings.get(host).expect("Host root"), b"host");
}
