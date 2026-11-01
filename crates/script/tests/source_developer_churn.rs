// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private developer arrays must collect with bounded runtime roots beside unrelated host owners."
)]
use crate::{
    array_read_support::number, source_array_write_support::State,
    source_developer_support::compile,
};
use cod4x_script::binding::LegacyDeveloperMode::Enabled;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelResource, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn one_hundred_thousand_private_developer_aliases_use_small_scratch_beside_host_aliases() {
    let module = compile(
        b"main(host){i=0;while(i<100000){/# a=host;n=a.size; #/i++;}return i;}",
        Enabled,
    );
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 1,
        max_object_slots: 1,
        max_array_values: 70_004,
        max_array_value_slots: 70_004,
        ..ObjectHeapBudget::default()
    });
    let host = state.objects.new_array_value(work).expect("Host").value;
    let data = state
        .objects
        .array_data(host, work)
        .expect("Host data")
        .value;
    state
        .objects
        .array_put_unique(host, ArrayKey::Index(0), number(7), &state.strings, work)
        .expect("Host payload");
    for _ in 0..70_000 {
        state
            .objects
            .copy_array_value(host, work)
            .expect("Host aliases");
    }
    let (result, heap) = state.run(
        &module,
        &[KernelValue::Array(host)],
        KernelCallBudget {
            max_steps: 4_000_000,
            ..KernelCallBudget::default()
        },
        HeapWorkBudget {
            max_scratch_bytes: 4096,
            ..work
        },
    );
    assert_eq!(
        result.expect("Bounded developer churn").value,
        number(100_000)
    );
    assert_eq!((heap.collections, heap.object_collections), (0, 0));
    assert!(heap.peak_scratch_bytes <= 4096);
    assert_eq!(state.objects.usage().object_slots, 1);
    assert_eq!(state.objects.usage().live_array_values, 70_001);
    assert_eq!(
        state
            .objects
            .array_owner_count(data, work)
            .expect("Only host owners")
            .value,
        70_001
    );
    assert_eq!(
        state
            .objects
            .array_get(data, ArrayKey::Index(0), &state.strings, work)
            .expect("Preserved host")
            .value,
        Some(number(7))
    );
}

#[test]
fn private_developer_construction_churn_collects_with_two_data_slots_and_bounded_scratch() {
    let module = compile(
        b"main(){i=0;while(i<100000){/# a=[];a[0]=1; #/i++;}return i;}",
        Enabled,
    );
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 2,
        max_object_slots: 2,
        max_array_values: 2,
        max_array_value_slots: 2,
        ..ObjectHeapBudget::default()
    });
    let (result, heap) = state.run(
        &module,
        &[],
        KernelCallBudget {
            max_steps: 4_000_000,
            ..KernelCallBudget::default()
        },
        HeapWorkBudget {
            max_scratch_bytes: 4096,
            ..HeapWorkBudget::default()
        },
    );
    assert_eq!(
        result.expect("Private constructors reclaim").value,
        number(100_000)
    );
    assert!(heap.collections >= 49_999);
    assert_eq!(heap.collections, heap.object_collections);
    assert!(heap.peak_scratch_bytes <= 4096);
    assert_eq!(state.objects.usage().object_slots, 2);
    assert_eq!(state.objects.usage().live_array_values, 0);
}

#[test]
fn collection_owner_audit_cost_remains_explicit_and_exhaustion_preserves_host_owners() {
    let module = compile(
        b"main(){i=0;while(i<10){/# a=[];a[0]=1; #/i++;}return i;}",
        Enabled,
    );
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 3,
        max_object_slots: 3,
        ..ObjectHeapBudget::default()
    });
    let host = state.objects.new_array_value(work).expect("Host").value;
    let data = state
        .objects
        .array_data(host, work)
        .expect("Host data")
        .value;
    state
        .objects
        .array_put_unique(host, ArrayKey::Index(0), number(7), &state.strings, work)
        .expect("Host graph");
    for _ in 0..70_000 {
        state
            .objects
            .copy_array_value(host, work)
            .expect("Host aliases");
    }
    let (result, heap) = state.run(
        &module,
        &[],
        KernelCallBudget::default(),
        HeapWorkBudget {
            max_steps: 64_000,
            max_scratch_bytes: 4096,
        },
    );
    assert!(matches!(
        result
            .expect_err("Complete owner audit is currently linear in retained locations")
            .cause
            .kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::HeapWorkSteps,
            ..
        }
    ));
    assert!(heap.steps <= 64_000);
    assert_eq!(state.objects.usage().live_array_values, 70_001);
    assert_eq!(
        state
            .objects
            .array_owner_count(data, work)
            .expect("Host only")
            .value,
        70_001
    );
    assert_eq!(
        state
            .objects
            .array_get(data, ArrayKey::Index(0), &state.strings, work)
            .expect("Host unchanged")
            .value,
        Some(number(7))
    );
}
