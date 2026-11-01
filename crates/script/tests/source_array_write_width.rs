// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Source arrays must retain wide storage and bounded alias/update scratch."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::KernelCallBudget;
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn a_seventy_thousand_byte_source_key_preserves_complete_identity_and_tagged_storage() {
    let key = "k".repeat(70_000);
    let source = format!("main(){{a=[];a[\"{key}\"]=9;return a[\"{key}\"]+a.size;}}");
    let module = compile(source.as_bytes());
    let mut state = State::new(ObjectHeapBudget {
        max_key_bytes: 70_001,
        max_single_key_bytes: 70_001,
        ..ObjectHeapBudget::default()
    });
    let result = state
        .run(
            &module,
            &[],
            KernelCallBudget::default(),
            HeapWorkBudget::default(),
        )
        .0
        .expect("Complete source key");
    assert_eq!(result.value, number(10));
    assert_eq!(state.objects.usage().key_bytes, 70_001);
    assert_eq!(state.objects.usage().live_array_values, 0);
}

#[test]
fn seventy_thousand_source_entries_retain_full_indices_and_count() {
    let module = compile(b"main(){a=[];for(i=0;i<70000;i++){a[i]=i;}return a.size+a[69999];}");
    let mut state = State::new(ObjectHeapBudget {
        max_live_fields: 70_000,
        max_field_slots: 70_000,
        max_live_objects: 1,
        max_object_slots: 1,
        max_array_values: 2,
        max_array_value_slots: 2,
        ..ObjectHeapBudget::default()
    });
    let result = state
        .run(
            &module,
            &[],
            KernelCallBudget {
                max_steps: 4_000_000,
                ..KernelCallBudget::default()
            },
            HeapWorkBudget::default(),
        )
        .0
        .expect("Wide source fields");
    assert_eq!(result.value, number(139_999));
    let usage = state.objects.usage();
    assert_eq!(
        (usage.live_fields, usage.field_slots, usage.key_bytes),
        (70_000, 70_000, 630_000)
    );
    assert_eq!(usage.live_array_values, 0);
    assert_eq!(
        state
            .objects
            .reclaim(&[], HeapWorkBudget::default())
            .expect("Iterative field reclamation")
            .fields,
        70_000
    );
}

#[test]
fn one_hundred_thousand_source_updates_and_private_aliases_use_four_kib_scratch_beside_host_owners()
{
    let module =
        compile(b"main(){a=[];i=0;while(i<100000){if(1){private=a;}a[0]=i;i++;}return a[0];}");
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 2,
        max_object_slots: 2,
        max_array_values: 70_003,
        max_array_value_slots: 70_003,
        max_live_fields: 2,
        max_field_slots: 2,
        max_key_bytes: 18,
        max_index_buckets: 8,
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
        .array_put_unique(host, ArrayKey::Index(1), number(77), &state.strings, work)
        .expect("Host payload");
    for _ in 0..70_000 {
        state
            .objects
            .copy_array_value(host, work)
            .expect("Unrelated host owner");
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
        result.expect("Private alias never forces a copy").value,
        number(99_999)
    );
    assert_eq!((heap.object_collections, heap.collections), (0, 0));
    assert!(heap.peak_scratch_bytes <= 4096);
    let usage = state.objects.usage();
    assert_eq!(
        (usage.live_arrays, usage.live_fields, usage.field_slots),
        (2, 2, 2)
    );
    assert_eq!(
        (usage.live_array_values, usage.array_value_slots),
        (70_001, 70_003)
    );
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
            .array_get(data, ArrayKey::Index(1), &state.strings, work)
            .expect("Host payload unchanged")
            .value,
        Some(number(77))
    );
}
