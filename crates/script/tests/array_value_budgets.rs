// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests admit owned array records exactly and reject independent quota failures atomically."
)]
use crate::object_heap_support::stores;
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind, ObjectResource};

#[test]
fn empty_array_value_creation_admits_exact_data_value_and_metadata_counts_without_partial_publication()
 {
    let (_, mut sample) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    sample
        .new_array_value(budget)
        .expect("Measure complete creation");
    let usage = sample.usage();
    let exact = ObjectHeapBudget {
        max_live_objects: 1,
        max_object_slots: 1,
        max_array_values: 1,
        max_array_value_slots: 1,
        max_metadata_bytes: usage.metadata_bytes,
        ..ObjectHeapBudget::default()
    };
    let (_, mut heap) = stores(exact);
    heap.new_array_value(budget)
        .expect("Exact all retained resources");
    assert_eq!(heap.usage(), usage);
    for (limits, resource) in [
        (
            ObjectHeapBudget {
                max_live_objects: 0,
                ..exact
            },
            ObjectResource::LiveObjects,
        ),
        (
            ObjectHeapBudget {
                max_object_slots: 0,
                ..exact
            },
            ObjectResource::ObjectSlots,
        ),
        (
            ObjectHeapBudget {
                max_array_values: 0,
                ..exact
            },
            ObjectResource::ArrayValues,
        ),
        (
            ObjectHeapBudget {
                max_array_value_slots: 0,
                ..exact
            },
            ObjectResource::ArrayValueSlots,
        ),
        (
            ObjectHeapBudget {
                max_metadata_bytes: exact.max_metadata_bytes - 1,
                ..exact
            },
            ObjectResource::MetadataBytes,
        ),
    ] {
        let (_, mut heap) = stores(limits);
        let before = heap.usage();
        let error = heap
            .new_array_value(budget)
            .expect_err("One less independent resource");
        assert!(
            matches!(error.kind, ObjectHeapErrorKind::BudgetExceeded {resource: actual, requested, maximum} if actual == resource && requested > maximum)
        );
        assert_eq!(heap.usage(), before);
    }
}

#[test]
fn failed_alias_and_detach_admission_preserve_owned_counts_locations_and_old_identity() {
    let budget = HeapWorkBudget::default();
    for limits in [
        ObjectHeapBudget {
            max_array_values: 1,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_array_value_slots: 1,
            ..ObjectHeapBudget::default()
        },
    ] {
        let (_, mut heap) = stores(limits);
        let first = heap.new_array_value(budget).expect("First owner").value;
        let data = heap.array_data(first, budget).expect("Original data").value;
        let before = heap.usage();
        assert!(heap.copy_array_value(first, budget).is_err());
        assert_eq!(heap.usage(), before);
        assert_eq!(
            heap.array_owner_count(data, budget)
                .expect("No partial retain")
                .value,
            1
        );
    }
    for limits in [
        ObjectHeapBudget {
            max_live_objects: 1,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_object_slots: 1,
            ..ObjectHeapBudget::default()
        },
    ] {
        let (_, mut heap) = stores(limits);
        let first = heap.new_array_value(budget).expect("Initial value").value;
        let alias = heap
            .copy_array_value(first, budget)
            .expect("Second value without another data node")
            .value;
        let data = heap.array_data(first, budget).expect("Shared data").value;
        let before = heap.usage();
        assert!(heap.make_empty_array_unique(alias, budget).is_err());
        assert_eq!(heap.usage(), before);
        assert_eq!(
            heap.array_data(alias, budget)
                .expect("No rewritten target")
                .value,
            data
        );
        assert_eq!(
            heap.array_owner_count(data, budget)
                .expect("No lost owner")
                .value,
            2
        );
    }
}
