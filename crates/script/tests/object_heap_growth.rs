// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests validate atomic growth under every work prefix."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectId, ObjectResource,
    StringHeap,
};

fn full_index() -> (StringHeap, ObjectHeap, ObjectId) {
    let (strings, mut objects) = stores(ObjectHeapBudget::default());
    let id = allocate(&mut objects);
    for key in 0_u8..4 {
        objects
            .put(
                id,
                &[key],
                number(i32::from(key)),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Existing field");
    }
    assert_eq!(objects.usage().index_buckets, 8);
    (strings, objects, id)
}

#[test]
fn object_index_growth_is_atomic_at_every_insufficient_work_prefix_and_scratch_limit() {
    let (strings, mut objects, id) = full_index();
    let usage = objects
        .put(id, b"new", number(9), &strings, HeapWorkBudget::default())
        .expect("Measure growth")
        .usage;
    assert_eq!(objects.usage().index_buckets, 16);
    let budgets = (0..usage.steps)
        .map(|max_steps| HeapWorkBudget {
            max_steps,
            ..HeapWorkBudget::default()
        })
        .chain([HeapWorkBudget {
            max_scratch_bytes: usage.scratch_bytes - 1,
            ..HeapWorkBudget::default()
        }]);
    for budget in budgets {
        let (strings, mut objects, id) = full_index();
        let before = objects.usage();
        let error = objects
            .put(id, b"new", number(9), &strings, budget)
            .expect_err("Exhaustion");
        assert!(matches!(
            error.kind,
            ObjectHeapErrorKind::BudgetExceeded {
                resource: ObjectResource::WorkSteps | ObjectResource::ScratchBytes,
                ..
            }
        ));
        assert!(error.usage.steps <= budget.max_steps);
        assert_eq!(objects.usage(), before);
        assert_eq!(objects.field_count(id), Ok(4));
        for key in 0_u8..4 {
            assert_eq!(
                objects
                    .get(id, &[key], &strings, HeapWorkBudget::default())
                    .expect("Preserved")
                    .value,
                Some(number(i32::from(key)))
            );
        }
        assert_eq!(
            objects
                .get(id, b"new", &strings, HeapWorkBudget::default())
                .expect("Absent")
                .value,
            None
        );
        objects
            .put(id, b"new", number(9), &strings, HeapWorkBudget::default())
            .expect("Retry");
        assert_eq!(objects.field_count(id), Ok(5));
    }
    let (strings, mut objects, id) = full_index();
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.scratch_bytes,
    };
    assert_eq!(
        objects
            .put(id, b"new", number(9), &strings, exact)
            .expect("Exact growth")
            .usage,
        usage
    );
}
