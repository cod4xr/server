// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust every work prefix before owned-value/data publication and collection."
)]
use crate::object_heap_support::stores;
use cod4x_script::heap::{
    ArrayCowOutcome, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget,
    ObjectHeapErrorKind, ObjectResource, StringHeap,
};

fn graph() -> (StringHeap, ObjectHeap, Vec<ArrayValueId>) {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let values = (0..4)
        .map(|_| {
            heap.new_array_value(HeapWorkBudget::default())
                .expect("Populate reservation boundaries")
                .value
        })
        .collect();
    (strings, heap, values)
}

#[test]
fn creation_with_existing_array_metadata_admits_exact_work_and_every_failure_preserves_all_owners()
{
    let (_, mut measured, _) = graph();
    let usage = measured
        .new_array_value(HeapWorkBudget::default())
        .expect("Measured creation")
        .usage;
    for maximum in 0..=usage.steps {
        let (_, mut heap, values) = graph();
        let before = heap.usage();
        let result = heap.new_array_value(HeapWorkBudget {
            max_steps: maximum,
            max_scratch_bytes: 0,
        });
        if maximum < usage.steps {
            let error = result.expect_err("Every insufficient publication prefix");
            assert!(
                matches!(error.kind, ObjectHeapErrorKind::BudgetExceeded {resource: ObjectResource::WorkSteps, requested, maximum: actual} if requested > actual && actual == maximum)
            );
            assert!(error.usage.steps <= maximum);
            assert_eq!(heap.usage(), before);
        } else {
            assert_eq!(result.expect("Exact creation").usage, usage);
        }
        for value in values {
            let data = heap
                .array_data(value, HeapWorkBudget::default())
                .expect("Existing identity remains valid")
                .value;
            assert_eq!(
                heap.array_owner_count(data, HeapWorkBudget::default())
                    .expect("Existing count remains valid")
                    .value,
                1
            );
        }
    }
}

#[test]
fn copy_detach_and_release_exhaust_every_insufficient_work_prefix_without_partial_mutation() {
    for operation in 0..3 {
        let (_, mut sample) = stores(ObjectHeapBudget::default());
        let value = sample
            .new_array_value(HeapWorkBudget::default())
            .expect("Value")
            .value;
        if operation == 1 {
            sample
                .copy_array_value(value, HeapWorkBudget::default())
                .expect("Shared data");
        }
        let usage = match operation {
            0 => {
                sample
                    .copy_array_value(value, HeapWorkBudget::default())
                    .expect("Copy")
                    .usage
            }
            1 => {
                sample
                    .make_empty_array_unique(value, HeapWorkBudget::default())
                    .expect("Detach")
                    .usage
            }
            _ => {
                sample
                    .release_array_value(value, HeapWorkBudget::default())
                    .expect("Release")
                    .usage
            }
        };
        for maximum in 0..=usage.steps {
            let (_, mut heap) = stores(ObjectHeapBudget::default());
            let value = heap
                .new_array_value(HeapWorkBudget::default())
                .expect("Value")
                .value;
            if operation == 1 {
                heap.copy_array_value(value, HeapWorkBudget::default())
                    .expect("Alias");
            }
            let data = heap
                .array_data(value, HeapWorkBudget::default())
                .expect("Old data")
                .value;
            let before = heap.usage();
            let budget = HeapWorkBudget {
                max_steps: maximum,
                max_scratch_bytes: 0,
            };
            let result = match operation {
                0 => heap
                    .copy_array_value(value, budget)
                    .map(|result| result.usage),
                1 => heap.make_empty_array_unique(value, budget).map(|result| {
                    assert_eq!(result.value, ArrayCowOutcome::Detached);
                    result.usage
                }),
                _ => heap
                    .release_array_value(value, budget)
                    .map(|result| result.usage),
            };
            if maximum < usage.steps {
                let error = result.expect_err("Incomplete operation");
                assert!(
                    matches!(error.kind, ObjectHeapErrorKind::BudgetExceeded {resource: ObjectResource::WorkSteps, requested, maximum: actual} if requested > actual && actual == maximum)
                );
                assert_eq!(heap.usage(), before);
                assert_eq!(
                    heap.array_data(value, HeapWorkBudget::default())
                        .expect("No rewritten/released target")
                        .value,
                    data
                );
            } else {
                assert_eq!(result.expect("Exact operation work"), usage);
            }
        }
    }
}
