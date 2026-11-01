// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Permuted source labels are checked against an independent direct lookup model."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use std::fmt::Write;

#[test]
fn all_permuted_mixed_case_keys_and_misses_match_direct_lookup_across_reused_contexts() {
    let mut source = String::from("main(k){switch(k){");
    // An odd permutation modulo 128 scrambles signed integers and strings alike.
    for ordinal in 0..128 {
        let key = (ordinal * 73 + 19) % 128 - 64;
        write!(
            source,
            "case \"k{key}\":return {};case {key}:return {};",
            key + 200,
            key + 400
        )
        .expect("Cases");
    }
    source.push_str("default:return -999;}}");
    let module = compile(source.as_bytes());
    assert_eq!(
        (module.usage().switch_labels, module.usage().string_literals),
        (128, 0)
    );
    for key in -66..66 {
        let mut strings = StringHeap::new(StringHeapBudget {
            max_live_strings: 1,
            max_slots: 1,
            ..StringHeapBudget::default()
        })
        .expect("One input");
        let input = format!("k{key}");
        let id = strings
            .allocate(input.as_bytes(), HeapWorkBudget::default())
            .expect("Input")
            .id;
        let mut context = KernelHeapContext::new(
            &mut strings,
            &[],
            HeapWorkBudget {
                max_scratch_bytes: 0,
                ..HeapWorkBudget::default()
            },
        );
        // A failed invocation must not poison either domain on this same context.
        module
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::Undefined],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect_err("Wrong type");
        for (value, offset) in [(number(key), 400), (KernelValue::String(id), 200)] {
            let expected = if (-64..64).contains(&key) {
                key + offset
            } else {
                -999
            };
            let result = module
                .program()
                .execute_with_heap(0, &[value], KernelCallBudget::default(), &mut context)
                .expect("Independent domain lookup");
            assert_eq!(result.value, number(expected));
            assert!(result.usage.steps < 90);
        }
        assert_eq!(
            (context.heap_usage().slots, context.usage().collections),
            (1, 0)
        );
    }
}

#[test]
fn one_hundred_thousand_source_dispatches_fit_small_scratch_beside_seventy_thousand_host_aliases() {
    let module = compile(br#"main(k){i=0;while(i<100000){switch(k){case "needle":i++;break;default:return -1;}}return i;}"#);
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 1,
        max_object_slots: 1,
        max_array_values: 70_001,
        max_array_value_slots: 70_001,
        ..ObjectHeapBudget::default()
    });
    let id = state.strings.allocate(b"needle", work).expect("Input").id;
    let host = state
        .objects
        .new_array_value(work)
        .expect("Unrelated host")
        .value;
    let data = state
        .objects
        .array_data(host, work)
        .expect("Host data")
        .value;
    state
        .objects
        .array_put_unique(
            host,
            ArrayKey::Index(0),
            KernelValue::String(id),
            &state.strings,
            work,
        )
        .expect("Host graph");
    for _ in 0..70_000 {
        state
            .objects
            .copy_array_value(host, work)
            .expect("Host aliases");
    }
    let (result, heap) = state.run(
        &module,
        &[KernelValue::String(id)],
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
        result.expect("Bounded source dispatch").value,
        number(100_000)
    );
    assert!(heap.steps < 1_000_000);
    assert!(heap.peak_scratch_bytes <= 4096);
    assert_eq!((heap.collections, heap.object_collections), (0, 0));
    assert_eq!(state.strings.usage().slots, 1);
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
        Some(KernelValue::String(id))
    );
}
