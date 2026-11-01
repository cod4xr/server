// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private audit tests inject malformed owner locations and verify witness minting and full-audit boundaries."
)]
use super::arrays::ArrayValueState;
use crate::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, StringHeap, StringHeapBudget,
};

fn heap() -> (StringHeap, ObjectHeap) {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    (strings, objects)
}

#[test]
fn public_collection_keeps_the_full_owner_audit_after_vm_witness_reuse() {
    let (_, mut heap) = heap();
    let work = HeapWorkBudget::default();
    let value = heap.new_array_value(work).expect("Host").value;
    for _ in 0..1000 {
        heap.copy_array_value(value, work).expect("Aliases");
    }
    let mut audit = None;
    let first = heap
        .reclaim_for_vm(&[], &mut audit, work)
        .expect("First full audit");
    let reused = heap
        .reclaim_for_vm(&[], &mut audit, work)
        .expect("Audited reuse");
    let public = heap
        .reclaim(&[], work)
        .expect("Public path remains fully checked");
    assert_eq!(first.usage.steps, public.usage.steps);
    assert!(reused.usage.steps + 4000 <= public.usage.steps);
    // This impossible-through-public-API fault must still reject on the public path.
    heap.arrays.metadata[0].owners = 0;
    let before = heap.usage();
    assert_eq!(
        heap.reclaim(&[], work)
            .expect_err("No public audit bypass")
            .kind,
        ObjectHeapErrorKind::InvalidState
    );
    assert_eq!(heap.usage(), before);
}

#[test]
fn every_shorter_initial_audit_budget_preserves_state_and_mints_no_witness() {
    let work = HeapWorkBudget::default();
    let (_, mut baseline) = heap();
    let value = baseline.new_array_value(work).expect("Host").value;
    for _ in 0..3 {
        baseline.copy_array_value(value, work).expect("Aliases");
    }
    baseline.allocate(work).expect("Garbage");
    let mut audit = None;
    let exact = baseline
        .reclaim_for_vm(&[], &mut audit, work)
        .expect("Baseline")
        .usage;
    for maximum in 0..exact.steps {
        let (_, mut heap) = heap();
        let value = heap.new_array_value(work).expect("Host").value;
        for _ in 0..3 {
            heap.copy_array_value(value, work).expect("Aliases");
        }
        heap.allocate(work).expect("Garbage");
        let before = heap.usage();
        let mut audit = None;
        let budget = HeapWorkBudget {
            max_steps: maximum,
            max_scratch_bytes: exact.scratch_bytes,
        };
        heap.reclaim_for_vm(&[], &mut audit, budget)
            .expect_err("Every incomplete scan/mark/preflight/sweep prefix");
        assert!(audit.is_none());
        assert_eq!(heap.usage(), before);
    }
}

#[test]
fn a_foreign_witness_cannot_skip_initial_location_validation_or_sweep_malformed_state() {
    let (_, mut first) = heap();
    let (_, mut second) = heap();
    let work = HeapWorkBudget::default();
    let foreign = first.allocate(work).expect("Foreign object").id;
    let mut audit = None;
    first
        .reclaim_for_vm(&[foreign], &mut audit, work)
        .expect("First store audit");
    second.new_array_value(work).expect("Second store host");
    second.allocate(work).expect("Second store garbage");
    second.arrays.values[0].state = ArrayValueState::Occupied(foreign);
    let before = second.usage();
    assert_eq!(
        second
            .reclaim_for_vm(&[], &mut audit, work)
            .expect_err("Foreign witness forces full validation")
            .kind,
        ObjectHeapErrorKind::ForeignReference
    );
    assert_eq!(second.usage(), before);
    assert!(audit.as_ref().expect("Original witness").belongs_to(&first));
    assert!(
        !audit
            .as_ref()
            .expect("Original witness")
            .belongs_to(&second)
    );
}

#[test]
fn an_empty_store_audit_survives_checked_new_copy_cow_release_and_collection_operations() {
    let (strings, mut heap) = heap();
    let work = HeapWorkBudget::default();
    let mut audit = None;
    heap.reclaim_for_vm(&[], &mut audit, work)
        .expect("Empty store proof");
    let first = heap.new_array_value(work).expect("New data").value;
    let second = heap.copy_array_value(first, work).expect("Alias").value;
    heap.make_empty_array_unique(second, work)
        .expect("Empty COW");
    heap.array_put_unique(
        second,
        crate::heap::ArrayKey::Index(0),
        crate::execution::KernelValue::Undefined,
        &strings,
        work,
    )
    .expect("Populate");
    let third = heap
        .copy_array_value(second, work)
        .expect("Populated alias")
        .value;
    heap.make_array_unique(third, &strings, work)
        .expect("Populated COW");
    heap.release_array_value(first, work)
        .expect("Release first");
    heap.release_array_value(second, work)
        .expect("Release second");
    let reused = heap
        .reclaim_for_vm(&[], &mut audit, work)
        .expect("Checked mutations preserve the witness");
    assert_eq!(reused.objects, 2);
    assert_eq!(heap.usage().live_array_values, 1);
    assert_eq!(
        heap.array_len(
            heap.array_data(third, work).expect("Last owner").value,
            work
        )
        .expect("Copied data survives")
        .value,
        1
    );
    heap.release_array_value(third, work).expect("Last release");
    heap.reclaim_for_vm(&[], &mut audit, work)
        .expect("Final cached sweep");
    assert_eq!(heap.usage().live_arrays, 0);
}

#[test]
fn every_shorter_cached_collection_preserves_the_witness_and_atomic_retry() {
    let work = HeapWorkBudget::default();
    let setup = || {
        let (_, mut heap) = heap();
        let value = heap.new_array_value(work).expect("Host").value;
        let mut audit = None;
        heap.reclaim_for_vm(&[], &mut audit, work)
            .expect("Initial complete audit");
        heap.allocate(work).expect("Garbage after audit");
        (heap, value, audit)
    };
    let (mut baseline, _, mut audit) = setup();
    let exact = baseline
        .reclaim_for_vm(&[], &mut audit, work)
        .expect("Cached baseline")
        .usage;
    for maximum in 0..exact.steps {
        let (mut heap, value, mut audit) = setup();
        let before = heap.usage();
        let budget = HeapWorkBudget {
            max_steps: maximum,
            max_scratch_bytes: exact.scratch_bytes,
        };
        heap.reclaim_for_vm(&[], &mut audit, budget)
            .expect_err("Every shorter cached prefix");
        assert_eq!(heap.usage(), before);
        assert!(audit.as_ref().expect("Existing proof").belongs_to(&heap));
        assert_eq!(
            heap.reclaim_for_vm(&[], &mut audit, work)
                .expect("Same store retry")
                .objects,
            1
        );
        assert!(heap.array_data(value, work).is_ok());
    }
    let (mut heap, _, mut audit) = setup();
    let before = heap.usage();
    heap.reclaim_for_vm(
        &[],
        &mut audit,
        HeapWorkBudget {
            max_steps: exact.steps,
            max_scratch_bytes: exact.scratch_bytes - 1,
        },
    )
    .expect_err("One less mark workspace");
    assert_eq!(heap.usage(), before);
}
