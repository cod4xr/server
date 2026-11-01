// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests distinguish cycle members, their garbage closure and unique live boundary targets."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn unreachable_cycle_closure_includes_downstream_garbage_and_excludes_acyclic_upstream_nodes() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let root = allocate(&mut heap);
    let cycle = allocate(&mut heap);
    let downstream = allocate(&mut heap);
    let upstream = allocate(&mut heap);
    let tree = allocate(&mut heap);
    for (owner, key, target) in [
        (cycle, b"self".as_slice(), cycle),
        (cycle, b"tail", downstream),
        (downstream, b"live", root),
        (downstream, b"also-live", root),
        (upstream, b"cycle", cycle),
        (upstream, b"live", root),
        (tree, b"upstream", upstream),
    ] {
        heap.put(
            owner,
            key,
            KernelValue::Object(target),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Graph edge");
    }
    heap.put(
        root,
        b"PrivatePayload",
        number(1),
        &strings,
        HeapWorkBudget::default(),
    )
    .expect("Unexposed payload");
    let before = heap.usage();
    let result = heap
        .analyze_cycles(&[root, root], HeapWorkBudget::default())
        .expect("Distinct cycle closure");
    assert_eq!(
        (
            result.unreachable_objects,
            result.cycle_retained_objects,
            result.cycle_to_live_edges
        ),
        (4, 2, 2)
    );
    assert_eq!(result.live_targets(), &[root]);
    assert!(!format!("{result:?}").contains("PrivatePayload"));
    assert!(!format!("{result:?}").contains("ObjectId"));
    assert_eq!(heap.usage(), before);
    let reachable = heap
        .analyze_cycles(&[tree], HeapWorkBudget::default())
        .expect("Root reaches the complete cycle graph");
    assert_eq!(
        (
            reachable.unreachable_objects,
            reachable.cycle_retained_objects,
            reachable.cycle_to_live_edges
        ),
        (0, 0, 0)
    );
    assert!(reachable.live_targets().is_empty());
    assert_eq!(
        heap.reclaim(&[], HeapWorkBudget::default())
            .expect("Reports do not root their target snapshots")
            .objects,
        5
    );
    assert_eq!(
        heap.field_count(result.live_targets()[0]),
        Err(ObjectHeapErrorKind::StaleReference)
    );
}

#[test]
fn cycle_analysis_handles_empty_vacant_reused_foreign_and_stale_roots() {
    let (_, mut heap) = stores(ObjectHeapBudget::default());
    let empty = heap
        .analyze_cycles(
            &[],
            HeapWorkBudget {
                max_steps: 0,
                max_scratch_bytes: 0,
            },
        )
        .expect("Empty analysis needs no work/storage");
    assert_eq!(
        (
            empty.unreachable_objects,
            empty.usage.steps,
            empty.usage.scratch_bytes
        ),
        (0, 0, 0)
    );
    let dead = allocate(&mut heap);
    heap.reclaim(&[], HeapWorkBudget::default())
        .expect("Vacate a generation");
    let vacant = heap
        .analyze_cycles(&[], HeapWorkBudget::default())
        .expect("Vacant slots are not objects");
    assert_eq!(
        (vacant.unreachable_objects, vacant.cycle_retained_objects),
        (0, 0)
    );
    let live = allocate(&mut heap);
    let (_, mut other) = stores(ObjectHeapBudget::default());
    let foreign = allocate(&mut other);
    for (root, expected) in [
        (dead, ObjectHeapErrorKind::StaleReference),
        (foreign, ObjectHeapErrorKind::ForeignReference),
    ] {
        assert_eq!(
            heap.analyze_cycles(&[root], HeapWorkBudget::default())
                .expect_err("Checked roots")
                .kind,
            expected
        );
    }
    assert_eq!(
        heap.analyze_cycles(&[live], HeapWorkBudget::default())
            .expect("Reused slot remains independently checked")
            .unreachable_objects,
        0
    );
}
