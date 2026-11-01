// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests check preflight before every collection mutation."
)]
use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapWorkBudget, HeapWorkUsage, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectId,
    ObjectResource, StringHeap, StringId,
};

fn graph() -> (
    StringHeap,
    ObjectHeap,
    ObjectId,
    ObjectId,
    StringId,
    StringId,
) {
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let root = allocate(&mut objects);
    let garbage = allocate(&mut objects);
    let live = strings
        .allocate(b"live", HeapWorkBudget::default())
        .expect("Live string")
        .id;
    let dead = strings
        .allocate(b"dead", HeapWorkBudget::default())
        .expect("Garbage string")
        .id;
    objects
        .put(
            root,
            b"value",
            KernelValue::String(live),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Field");
    objects
        .put(
            root,
            b"self",
            KernelValue::Object(root),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Live cycle");
    objects
        .put(
            garbage,
            b"self",
            KernelValue::Object(garbage),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Dead cycle");
    (strings, objects, root, garbage, live, dead)
}

fn limits(usage: HeapWorkUsage) -> impl Iterator<Item = HeapWorkBudget> {
    (0..usage.steps)
        .map(|max_steps| HeapWorkBudget {
            max_steps,
            ..HeapWorkBudget::default()
        })
        .chain([HeapWorkBudget {
            max_scratch_bytes: usage.scratch_bytes - 1,
            ..HeapWorkBudget::default()
        }])
}

#[test]
fn object_collection_preflights_every_work_prefix_scratch_and_late_invalid_root() {
    let (_, mut objects, root, _, _, _) = graph();
    let usage = objects
        .reclaim(&[root, root], HeapWorkBudget::default())
        .expect("Measure")
        .usage;
    for budget in limits(usage) {
        let (strings, mut objects, root, garbage, live, _) = graph();
        let before = objects.usage();
        let error = objects
            .reclaim(&[root, root], budget)
            .expect_err("Exhausted");
        assert!(matches!(
            error.kind,
            ObjectHeapErrorKind::BudgetExceeded {
                resource: ObjectResource::WorkSteps | ObjectResource::ScratchBytes,
                ..
            }
        ));
        assert!(error.usage.steps <= budget.max_steps);
        assert_eq!(objects.usage(), before);
        assert_eq!(objects.field_count(garbage), Ok(1));
        assert_eq!(
            objects
                .get(root, b"value", &strings, HeapWorkBudget::default())
                .expect("Preserved")
                .value,
            Some(KernelValue::String(live))
        );
        assert_eq!(
            objects
                .reclaim(&[root], HeapWorkBudget::default())
                .expect("Retry")
                .objects,
            1
        );
    }
    let (_, mut objects, root, garbage, _, _) = graph();
    let (_, mut foreign) = stores(ObjectHeapBudget::default());
    let invalid = allocate(&mut foreign);
    let before = objects.usage();
    assert_eq!(
        objects
            .reclaim(&[root, garbage, invalid], HeapWorkBudget::default())
            .expect_err("Late foreign root")
            .kind,
        ObjectHeapErrorKind::ForeignReference
    );
    assert_eq!(objects.usage(), before);
    let result = objects
        .reclaim(
            &[root, root],
            HeapWorkBudget {
                max_steps: usage.steps,
                max_scratch_bytes: usage.scratch_bytes,
            },
        )
        .expect("Exact admission");
    assert_eq!(result.usage, usage);
    assert_eq!((result.objects, result.fields), (1, 1));
}

#[test]
fn object_string_collection_shares_global_work_and_simultaneous_root_mark_scratch_atomically() {
    let (mut strings, objects, _, _, live, _) = graph();
    let usage = objects
        .reclaim_strings(&mut strings, &[live, live], HeapWorkBudget::default())
        .expect("Measure")
        .usage;
    for budget in limits(usage) {
        let (mut strings, objects, root, _, live, dead) = graph();
        let before = strings.usage();
        let object_before = objects.usage();
        let error = objects
            .reclaim_strings(&mut strings, &[live, live], budget)
            .expect_err("Combined limit");
        assert!(matches!(
            error.kind,
            ObjectHeapErrorKind::BudgetExceeded {
                resource: ObjectResource::WorkSteps | ObjectResource::ScratchBytes,
                ..
            }
        ));
        assert!(error.usage.steps <= budget.max_steps);
        assert_eq!(strings.usage(), before);
        assert_eq!(objects.usage(), object_before);
        assert_eq!(strings.get(dead).expect("No partial sweep"), b"dead");
        assert_eq!(
            objects
                .get(root, b"value", &strings, HeapWorkBudget::default())
                .expect("No stale stored field")
                .value,
            Some(KernelValue::String(live))
        );
    }
    let (mut strings, objects, _, _, live, dead) = graph();
    let result = objects
        .reclaim_strings(
            &mut strings,
            &[live, live],
            HeapWorkBudget {
                max_steps: usage.steps,
                max_scratch_bytes: usage.scratch_bytes,
            },
        )
        .expect("Exact composite budget");
    assert_eq!(result.usage, usage);
    assert_eq!(result.strings, 1);
    assert!(strings.get(dead).is_err());
}

#[test]
fn object_string_collection_rejects_wrong_store_and_late_stale_or_foreign_roots_without_sweep() {
    let (mut strings, objects, _, _, live, dead) = graph();
    let (mut foreign, _) = stores(ObjectHeapBudget::default());
    let foreign_id = foreign
        .allocate(b"foreign", HeapWorkBudget::default())
        .expect("Foreign string")
        .id;
    let before = strings.usage();
    assert!(matches!(
        objects
            .reclaim_strings(&mut strings, &[live, foreign_id], HeapWorkBudget::default())
            .expect_err("Foreign late root")
            .kind,
        ObjectHeapErrorKind::StringHeap(_)
    ));
    assert_eq!(strings.usage(), before);
    let foreign_before = foreign.usage();
    assert_eq!(
        objects
            .reclaim_strings(&mut foreign, &[], HeapWorkBudget::default())
            .expect_err("Wrong bound store")
            .kind,
        ObjectHeapErrorKind::ForeignStringHeap
    );
    assert_eq!(foreign.usage(), foreign_before);
    strings
        .reclaim(&[live], HeapWorkBudget::default())
        .expect("Create stale extra root");
    let before = strings.usage();
    assert!(matches!(
        objects
            .reclaim_strings(&mut strings, &[live, dead], HeapWorkBudget::default())
            .expect_err("Late stale root")
            .kind,
        ObjectHeapErrorKind::StringHeap(_)
    ));
    assert_eq!(strings.usage(), before);
}
