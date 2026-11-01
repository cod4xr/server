// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure finite exact string-store admission and deliberate exhaustion."
)]

use cod4x_script::heap::{
    HeapErrorKind, HeapResource, HeapWorkBudget, StringHeap, StringHeapBudget,
};

#[test]
fn every_retained_and_copy_work_limit_accepts_exact_and_rejects_one_less() -> Result<(), String> {
    let mut sample = StringHeap::new(StringHeapBudget::default()).expect("Measure records");
    sample
        .allocate(b"abc", HeapWorkBudget::default())
        .expect("Measure payload");
    let exact = StringHeapBudget {
        max_live_strings: 1,
        max_payload_bytes: 3,
        max_slots: 1,
        max_metadata_bytes: sample.usage().metadata_bytes,
        max_single_string_bytes: 3,
    };
    let work = HeapWorkBudget {
        max_steps: 4,
        max_scratch_bytes: 0,
    };
    for resource in [
        HeapResource::LiveStrings,
        HeapResource::PayloadBytes,
        HeapResource::Slots,
        HeapResource::MetadataBytes,
        HeapResource::SingleStringBytes,
    ] {
        let mut budget = exact;
        match resource {
            HeapResource::LiveStrings => budget.max_live_strings -= 1,
            HeapResource::PayloadBytes => budget.max_payload_bytes -= 1,
            HeapResource::Slots => budget.max_slots -= 1,
            HeapResource::MetadataBytes => budget.max_metadata_bytes -= 1,
            HeapResource::SingleStringBytes => budget.max_single_string_bytes -= 1,
            _ => return Err("Unexpected retained resource".into()),
        }
        let mut heap = StringHeap::new(budget).expect("Bounded store");
        let error = heap
            .allocate(b"abc", work)
            .expect_err("One less retained resource");
        assert!(
            matches!(error.kind, HeapErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert_eq!(heap.usage().live_strings, 0);
        assert_eq!(heap.usage().slots, 0);
    }
    for maximum in 0..4 {
        let mut heap = StringHeap::new(exact).expect("Exact store");
        let error = heap
            .allocate(
                b"abc",
                HeapWorkBudget {
                    max_steps: maximum,
                    ..work
                },
            )
            .expect_err("Copy work admitted before allocation");
        assert!(matches!(
            error.kind,
            HeapErrorKind::BudgetExceeded {
                resource: HeapResource::WorkSteps,
                ..
            }
        ));
        assert!(error.usage.steps <= maximum);
        assert_eq!(heap.usage().live_strings, 0);
    }
    let mut heap = StringHeap::new(exact).expect("Exact store");
    let allocation = heap.allocate(b"abc", work).expect("Exact admission");
    assert_eq!(allocation.usage.steps, 4);
    heap.reclaim(&[], HeapWorkBudget::default())
        .expect("Release under unchanged slot quota");
    let replacement = heap
        .allocate(b"xyz", work)
        .expect("Reuse does not need new slot metadata")
        .id;
    assert_eq!(heap.usage().slots, 1);
    assert_eq!(heap.get(replacement).expect("Replacement"), b"xyz");
    assert_eq!(heap.get(allocation.id), Err(HeapErrorKind::StaleReference));
    Ok(())
}

#[test]
fn failed_concat_or_zero_byte_count_exhaustion_preserves_existing_values() {
    let mut heap = StringHeap::new(StringHeapBudget {
        max_payload_bytes: 5,
        max_single_string_bytes: 6,
        ..StringHeapBudget::default()
    })
    .expect("Store");
    let root = heap
        .allocate(b"abc", HeapWorkBudget::default())
        .expect("Source")
        .id;
    let before = heap.usage();
    let error = heap
        .concatenate(root, root, HeapWorkBudget::default())
        .expect_err("Complete output plus live inputs admitted");
    assert_eq!(
        error.kind,
        HeapErrorKind::BudgetExceeded {
            resource: HeapResource::PayloadBytes,
            requested: 9,
            maximum: 5
        }
    );
    assert_eq!(heap.usage(), before);
    assert_eq!(heap.get(root).expect("Unchanged input"), b"abc");
    let mut empty = StringHeap::new(StringHeapBudget {
        max_live_strings: 0,
        max_payload_bytes: 0,
        max_single_string_bytes: 0,
        ..StringHeapBudget::default()
    })
    .expect("Empty-only byte allowance");
    assert_eq!(
        empty
            .allocate(&[], HeapWorkBudget::default())
            .expect_err("Empty payloads still cost values")
            .kind,
        HeapErrorKind::BudgetExceeded {
            resource: HeapResource::LiveStrings,
            requested: 1,
            maximum: 0
        }
    );
    assert_eq!(empty.usage().slots, 0);
}
