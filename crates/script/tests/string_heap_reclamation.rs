// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests deliberately exhaust root validation and each collection admission phase."
)]

use cod4x_script::heap::{
    HeapErrorKind, HeapResource, HeapWorkBudget, StringHeap, StringHeapBudget,
};

#[test]
fn failed_root_validation_and_work_admission_preserve_all_values_and_usage() {
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Store");
    let keep = heap
        .allocate(b"keep", HeapWorkBudget::default())
        .expect("Root")
        .id;
    let other = heap
        .allocate(b"other", HeapWorkBudget::default())
        .expect("Other")
        .id;
    let mut foreign = StringHeap::new(StringHeapBudget::default()).expect("Foreign");
    let foreign = foreign
        .allocate(b"foreign", HeapWorkBudget::default())
        .expect("Foreign value")
        .id;
    let before = heap.usage();
    let error = heap
        .reclaim(&[keep, foreign], HeapWorkBudget::default())
        .expect_err("Every root validates before deletion");
    assert_eq!(error.kind, HeapErrorKind::ForeignReference);
    assert_eq!(error.usage.steps, 4);
    assert_eq!(heap.usage(), before);
    for maximum in 0..7 {
        let error = heap
            .reclaim(
                &[keep],
                HeapWorkBudget {
                    max_steps: maximum,
                    ..HeapWorkBudget::default()
                },
            )
            .expect_err("Work exhausted before mutation");
        assert!(matches!(
            error.kind,
            HeapErrorKind::BudgetExceeded {
                resource: HeapResource::WorkSteps,
                ..
            }
        ));
        assert!(error.usage.steps <= maximum);
        assert_eq!(heap.usage(), before);
        assert_eq!(
            heap.get(other)
                .expect("Unrooted value not prematurely freed"),
            b"other"
        );
    }
    let error = heap
        .reclaim(
            &[keep],
            HeapWorkBudget {
                max_scratch_bytes: 1,
                ..HeapWorkBudget::default()
            },
        )
        .expect_err("Complete mark payload admitted first");
    assert!(matches!(
        error.kind,
        HeapErrorKind::BudgetExceeded {
            resource: HeapResource::ScratchBytes,
            requested: 2,
            maximum: 1
        }
    ));
    assert_eq!(heap.usage(), before);
    let outcome = heap
        .reclaim(
            &[keep],
            HeapWorkBudget {
                max_steps: 7,
                max_scratch_bytes: 2,
            },
        )
        .expect("Exact work/scratch");
    assert_eq!(outcome.usage.steps, 7);
    assert_eq!(outcome.freed_strings, 1);
    assert_eq!(outcome.freed_payload_bytes, 5);
    assert_eq!(heap.get(other), Err(HeapErrorKind::StaleReference));
    assert_eq!(heap.get(keep).expect("Root retained"), b"keep");
    let before = heap.usage();
    assert_eq!(
        heap.reclaim(&[other], HeapWorkBudget::default())
            .expect_err("Stale roots reject atomically")
            .kind,
        HeapErrorKind::StaleReference
    );
    assert_eq!(heap.usage(), before);
}

#[test]
fn duplicate_roots_empty_stores_and_repeated_reclamation_keep_exact_accounting() {
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Store");
    let empty = heap
        .reclaim(
            &[],
            HeapWorkBudget {
                max_steps: 0,
                max_scratch_bytes: 0,
            },
        )
        .expect("No fake work/storage");
    assert_eq!(empty.usage.steps, 0);
    let value = heap
        .allocate(b"x", HeapWorkBudget::default())
        .expect("Value")
        .id;
    let rooted = heap
        .reclaim(&[value, value], HeapWorkBudget::default())
        .expect("Duplicate roots valid");
    assert_eq!(rooted.freed_strings, 0);
    assert_eq!(rooted.usage.steps, 5);
    assert_eq!(
        heap.reclaim(&[], HeapWorkBudget::default())
            .expect("Free once")
            .freed_strings,
        1
    );
    assert_eq!(
        heap.reclaim(&[], HeapWorkBudget::default())
            .expect("No double free")
            .freed_strings,
        0
    );
    assert_eq!(heap.usage().live_strings, 0);
    assert_eq!(heap.usage().payload_bytes, 0);
    assert_eq!(heap.usage().slots, 1);
}
