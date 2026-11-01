// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust mixed-graph collection admission before any sweep publication."
)]
use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    ArrayId, ArrayKey, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectId,
    StringHeap,
};

fn graph() -> (StringHeap, ObjectHeap, ArrayValueId, ArrayId, ObjectId) {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap
        .new_array_value(budget)
        .expect("Live managed root")
        .value;
    let garbage = heap.new_array_value(budget).expect("Unowned garbage").value;
    let dead_data = heap.array_data(garbage, budget).expect("Snapshot").value;
    for target in [value, garbage] {
        heap.array_put_unique(
            target,
            ArrayKey::Index(0),
            KernelValue::Undefined,
            &strings,
            budget,
        )
        .expect("Live and garbage indexed entries");
    }
    heap.release_array_value(garbage, budget)
        .expect("Leave vacant root slot");
    let cycle = allocate(&mut heap);
    heap.put(cycle, b"self", KernelValue::Object(cycle), &strings, budget)
        .expect("Unrooted struct cycle");
    (strings, heap, value, dead_data, cycle)
}

#[test]
fn mixed_array_struct_collection_preflights_every_work_prefix_and_exact_scratch() {
    let (_, mut sample, _, _, _) = graph();
    let usage = sample
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Measure")
        .usage;
    for maximum in 0..=usage.steps {
        let (_, mut heap, value, dead_data, cycle) = graph();
        let before = heap.usage();
        let budget = HeapWorkBudget {
            max_steps: maximum,
            max_scratch_bytes: usage.scratch_bytes,
        };
        let result = heap.reclaim(&[], budget);
        if maximum < usage.steps {
            let error = result.expect_err("Every insufficient work prefix");
            assert!(error.usage.steps <= maximum);
            assert_eq!(heap.usage(), before);
            assert_eq!(heap.field_count(cycle), Ok(1));
            assert_eq!(
                heap.array_len(dead_data, HeapWorkBudget::default())
                    .expect("No partial sweep")
                    .value,
                1
            );
        } else {
            let collection = result.expect("Exact work and scratch");
            assert_eq!(collection.usage, usage);
            assert_eq!((collection.objects, collection.fields), (2, 2));
        }
        let data = heap
            .array_data(value, HeapWorkBudget::default())
            .expect("Owned root preserved")
            .value;
        assert_eq!(
            heap.array_owner_count(data, HeapWorkBudget::default())
                .expect("Count unchanged")
                .value,
            1
        );
    }
    let (_, mut heap, _, _, _) = graph();
    let before = heap.usage();
    assert!(
        heap.reclaim(
            &[],
            HeapWorkBudget {
                max_scratch_bytes: usage.scratch_bytes - 1,
                ..HeapWorkBudget::default()
            }
        )
        .is_err()
    );
    assert_eq!(heap.usage(), before);
}
