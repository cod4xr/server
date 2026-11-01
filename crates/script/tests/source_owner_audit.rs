// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "A single complete owner audit must make repeated GC scale with distinct data rather than unchanged host aliases."
)]
use crate::{
    array_read_support::number, source_array_write_support::State,
    source_developer_support::compile,
};
use cod4x_script::binding::LegacyDeveloperMode::Enabled;
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapErrorKind, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn a_hundred_thousand_constructor_iterations_collect_beside_seventy_thousand_host_aliases_after_one_full_audit()
 {
    let module = compile(
        b"main(){i=0;while(i<100000){/# a=[];a[0]=1; #/i++;}return i;}",
        Enabled,
    );
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 3,
        max_object_slots: 3,
        max_array_values: 70_002,
        max_array_value_slots: 70_002,
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
        result
            .expect("Unchanged shared work budget now fits repeated collection")
            .value,
        number(100_000)
    );
    assert!(heap.collections >= 49_999);
    assert_eq!(heap.collections, heap.object_collections);
    assert_eq!(heap.array_owner_audits, 1);
    assert_eq!(heap.array_owner_audit_reuses, heap.object_collections - 1);
    assert!(heap.peak_scratch_bytes <= 4096);
    assert_eq!(state.objects.usage().object_slots, 3);
    assert_eq!(state.objects.usage().live_array_values, 70_001);
    assert_eq!(
        state
            .objects
            .array_owner_count(data, work)
            .expect("Host owners unchanged")
            .value,
        70_001
    );
    assert_eq!(
        state
            .objects
            .array_get(data, ArrayKey::Index(0), &state.strings, work)
            .expect("Host payload unchanged")
            .value,
        Some(number(7))
    );
}

#[test]
fn cached_owned_data_roots_keep_host_only_struct_cycles_and_uncached_string_payloads_alive() {
    let module = compile(
        b"main(){i=0;while(i<1000){a=[];a[0]=i;a=undefined;i++;}return i;}",
        Enabled,
    );
    assert_eq!(module.usage().string_literals, 0);
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 4,
        max_object_slots: 4,
        max_array_values: 2,
        max_array_value_slots: 2,
        ..ObjectHeapBudget::default()
    });
    let string = state
        .strings
        .allocate(b"OnlyGraph", work)
        .expect("No source/cache backing")
        .id;
    let host = state
        .objects
        .new_array_value(work)
        .expect("Host root location")
        .value;
    let child = state.objects.allocate(work).expect("Shared child").id;
    state
        .objects
        .put(
            child,
            b"self",
            KernelValue::Object(child),
            &state.strings,
            work,
        )
        .expect("Cycle");
    state
        .objects
        .put(
            child,
            b"text",
            KernelValue::String(string),
            &state.strings,
            work,
        )
        .expect("Only payload root");
    state
        .objects
        .array_put_unique(
            host,
            ArrayKey::Index(0),
            KernelValue::Object(child),
            &state.strings,
            work,
        )
        .expect("Host graph edge");
    let (result, heap) = state.run(
        &module,
        &[],
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    assert_eq!(
        result
            .expect("Transitive roots survive reused audits")
            .value,
        number(1000)
    );
    assert!(heap.collections >= 499);
    assert_eq!(heap.array_owner_audits, 1);
    assert_eq!(heap.array_owner_audit_reuses, heap.object_collections - 1);
    assert_eq!(state.strings.get(string), Ok(b"OnlyGraph".as_slice()));
    assert_eq!(
        state
            .objects
            .get(child, b"self", &state.strings, work)
            .expect("Cycle still present")
            .value,
        Some(KernelValue::Object(child))
    );
    assert_eq!(state.objects.usage().live_array_values, 1);
    state
        .objects
        .release_array_value(host, work)
        .expect("Last host root release");
    state
        .objects
        .reclaim(&[], work)
        .expect("Public full audit and unrooted graph sweep");
    state
        .objects
        .reclaim_strings(&mut state.strings, &[], work)
        .expect("Unrooted string sweep");
    assert_eq!(state.objects.usage().live_objects, 0);
    assert_eq!(
        state.strings.get(string),
        Err(HeapErrorKind::StaleReference)
    );
}
