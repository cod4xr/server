// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::expect_used, reason = "Tests check exact resource rejection.")]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeapBudget, ObjectHeapError, ObjectHeapErrorKind, ObjectResource,
};

fn resource(error: ObjectHeapError, expected: ObjectResource) {
    assert!(
        matches!(error.kind, ObjectHeapErrorKind::BudgetExceeded { resource, .. } if resource == expected)
    );
}

fn exact_budget(usage: cod4x_script::heap::ObjectHeapUsage) -> ObjectHeapBudget {
    ObjectHeapBudget {
        max_live_objects: usage.live_objects,
        max_live_fields: usage.live_fields,
        max_key_bytes: usage.key_bytes,
        max_object_slots: usage.object_slots,
        max_field_slots: usage.field_slots,
        max_index_buckets: usage.index_buckets,
        max_metadata_bytes: usage.metadata_bytes,
        max_single_key_bytes: 3,
    }
}

#[test]
fn object_storage_admits_exact_counts_and_rejects_each_one_less_without_publication() {
    let work = HeapWorkBudget::default();
    let (strings, mut objects) = stores(ObjectHeapBudget::default());
    let id = allocate(&mut objects);
    objects
        .put(id, b"key", number(7), &strings, work)
        .expect("Measure");
    let usage = objects.usage();
    let exact = exact_budget(usage);
    let (strings, mut objects) = stores(exact);
    let id = allocate(&mut objects);
    objects
        .put(id, b"key", number(7), &strings, work)
        .expect("Exact");
    assert_eq!(objects.usage(), usage);
    let before = objects.usage();
    objects
        .put(id, b"key", number(8), &strings, work)
        .expect("Replace at capacity");
    assert_eq!(objects.usage(), before);
    for (expected, budget) in [
        (
            ObjectResource::LiveObjects,
            ObjectHeapBudget {
                max_live_objects: 0,
                ..exact
            },
        ),
        (
            ObjectResource::ObjectSlots,
            ObjectHeapBudget {
                max_object_slots: 0,
                ..exact
            },
        ),
    ] {
        let (_, mut objects) = stores(budget);
        let before = objects.usage();
        resource(
            objects.allocate(work).expect_err("One less object"),
            expected,
        );
        assert_eq!(objects.usage(), before);
    }
    let limits = [
        (
            ObjectResource::LiveFields,
            ObjectHeapBudget {
                max_live_fields: 0,
                ..exact
            },
        ),
        (
            ObjectResource::KeyBytes,
            ObjectHeapBudget {
                max_key_bytes: 2,
                ..exact
            },
        ),
        (
            ObjectResource::FieldSlots,
            ObjectHeapBudget {
                max_field_slots: 0,
                ..exact
            },
        ),
        (
            ObjectResource::IndexBuckets,
            ObjectHeapBudget {
                max_index_buckets: usage.index_buckets - 1,
                ..exact
            },
        ),
        (
            ObjectResource::MetadataBytes,
            ObjectHeapBudget {
                max_metadata_bytes: usage.metadata_bytes - 1,
                ..exact
            },
        ),
        (
            ObjectResource::SingleKeyBytes,
            ObjectHeapBudget {
                max_single_key_bytes: 2,
                ..exact
            },
        ),
    ];
    for (expected, budget) in limits {
        let (strings, mut objects) = stores(budget);
        let id = allocate(&mut objects);
        let before = objects.usage();
        resource(
            objects
                .put(id, b"key", number(7), &strings, work)
                .expect_err("One less field resource"),
            expected,
        );
        assert_eq!(objects.usage(), before);
        assert_eq!(objects.field_count(id), Ok(0));
    }
}
