// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Scopes must release exhausted owners without touching host aliases."
)]
use crate::{array_read_support::number, source_scope_support};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

#[test]
fn scoped_alias_churn_uses_small_scratch_beside_seventy_thousand_host_owners() {
    let module = source_scope_support::compile(
        b"main(){a=[];i=0;while(i<100000){if(i>=0){b=a;}i++;}c=[];return 0;}",
    );
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 4,
            max_object_slots: 4,
            max_array_values: 70_003,
            max_array_value_slots: 70_003,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Bounded stores");
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Host data").value;
    for _ in 0..70_000 {
        objects
            .copy_array_value(host, work)
            .expect("Unrelated host owner");
    }
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 4096,
            ..work
        },
    )
    .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget {
                max_steps: 4_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Private b cleared every iteration");
    assert_eq!(result.value, number(0));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (usage.live_array_values, usage.array_value_slots),
        (70_001, 70_003)
    );
    assert!(context.usage().peak_scratch_bytes <= 4096);
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (0, 0)
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Host owners unchanged")
            .value,
        70_001
    );
}

#[test]
fn repeated_private_aliases_and_constructors_collect_with_two_data_slots() {
    let module = source_scope_support::compile(
        b"main(){i=0;while(i<100000){a=[];if(1){b=a;}a=undefined;i++;}return 0;}",
    );
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 2,
            max_object_slots: 2,
            max_array_values: 2,
            max_array_value_slots: 2,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Bounded pressure stores");
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 4096,
            ..HeapWorkBudget::default()
        },
    )
    .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget {
                max_steps: 4_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Scoped alias releases before next constructor");
    assert_eq!(result.value, number(0));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.live_array_values,
            usage.array_value_slots,
            usage.object_slots
        ),
        (0, 2, 2)
    );
    assert!(context.usage().peak_scratch_bytes <= 4096);
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (49_999, 49_999)
    );
}

#[test]
fn repeated_failure_and_success_share_the_context_without_retained_scoped_owners() {
    let module = source_scope_support::compile(
        b"bad(){a=[];if(1){b=a;return 1/0;}}
        good(){a=[];if(1){b=a;}c=[];return 0;}",
    );
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_values: 2,
            max_array_value_slots: 2,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Two value slots");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    for _ in 0..32 {
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("Failure inside private branch");
        let result = module
            .program()
            .execute_with_heap(1, &[], KernelCallBudget::default(), &mut context)
            .expect("Same context recovery");
        assert_eq!(result.value, number(0));
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            0
        );
    }
}
