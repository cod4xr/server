// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure every failed analysis prefix and logical output overlap without graph mutation."
)]
use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectId, ObjectResource,
    StringHeap,
};

fn graph() -> (StringHeap, ObjectHeap, ObjectId) {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let root = allocate(&mut heap);
    for _ in 0..3 {
        let cycle = allocate(&mut heap);
        heap.put(
            cycle,
            b"self",
            KernelValue::Object(cycle),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Cycle");
        heap.put(
            cycle,
            b"root",
            KernelValue::Object(root),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Boundary");
    }
    (strings, heap, root)
}

#[test]
fn cycle_analysis_admits_exact_work_scratch_output_overlap_and_rejects_every_insufficient_prefix() {
    let (strings, heap, root) = graph();
    let before = heap.usage();
    let measured = heap
        .analyze_cycles(&[root], HeapWorkBudget::default())
        .expect("Measure");
    assert_eq!(measured.live_targets(), &[root]);
    assert_eq!(
        measured.usage.scratch_bytes,
        4 * (2 + 2 * size_of::<usize>()) as u64 + size_of::<ObjectId>() as u64
    );
    let exact = HeapWorkBudget {
        max_steps: measured.usage.steps,
        max_scratch_bytes: measured.usage.scratch_bytes,
    };
    let result = heap
        .analyze_cycles(&[root], exact)
        .expect("Exact workspace/output overlap");
    assert_eq!(result.usage, measured.usage);
    for budget in (0..exact.max_steps)
        .map(|max_steps| HeapWorkBudget { max_steps, ..exact })
        .chain([HeapWorkBudget {
            max_scratch_bytes: exact.max_scratch_bytes - 1,
            ..exact
        }])
    {
        let error = heap
            .analyze_cycles(&[root], budget)
            .expect_err("No partial analysis publication");
        assert!(
            matches!(error.kind, ObjectHeapErrorKind::BudgetExceeded {resource: ObjectResource::WorkSteps | ObjectResource::ScratchBytes, requested, maximum} if requested > maximum)
        );
        assert!(error.usage.steps <= budget.max_steps);
        assert!(error.usage.scratch_bytes <= budget.max_scratch_bytes);
        assert_eq!(heap.usage(), before);
    }
    assert_eq!(heap.field_count(root), Ok(0));
    assert_eq!(strings.usage().live_strings, 0);
}

#[test]
fn seventy_thousand_cycle_nodes_and_downstream_chain_analyze_without_recursion_or_short_counts() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let root = allocate(&mut heap);
    let first = allocate(&mut heap);
    let mut previous = first;
    for _ in 1..70_000 {
        let next = allocate(&mut heap);
        heap.put(
            previous,
            b"next",
            KernelValue::Object(next),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Wide graph edge");
        previous = next;
    }
    heap.put(
        previous,
        b"cycle",
        KernelValue::Object(first),
        &strings,
        HeapWorkBudget::default(),
    )
    .expect("Long cycle closes");
    let tail = allocate(&mut heap);
    heap.put(
        previous,
        b"tail",
        KernelValue::Object(tail),
        &strings,
        HeapWorkBudget::default(),
    )
    .expect("Downstream closure");
    heap.put(
        tail,
        b"live",
        KernelValue::Object(root),
        &strings,
        HeapWorkBudget::default(),
    )
    .expect("Live boundary");
    let result = heap
        .analyze_cycles(&[root], HeapWorkBudget::default())
        .expect("Iterative wide analysis");
    assert_eq!(
        (
            result.unreachable_objects,
            result.cycle_retained_objects,
            result.cycle_to_live_edges
        ),
        (70_001, 70_001, 1)
    );
    assert_eq!(result.live_targets(), &[root]);
    assert_eq!(
        heap.reclaim(&[root], HeapWorkBudget::default())
            .expect("Existing collector still reclaims cycles explicitly")
            .objects,
        70_001
    );
}
