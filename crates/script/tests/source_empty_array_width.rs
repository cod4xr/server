// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests independent source data above packed native limits and bounded constructor churn under pressure."
)]
use std::fmt::Write;

use crate::array_read_support::number;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::syntax::LegacySyntaxBudget;

#[test]
fn seventy_thousand_independent_source_arrays_and_owner_rows_remain_wide_and_reclaim_iteratively() {
    let mut source = String::from("main(){");
    for index in 0..70_000 {
        write!(source, "a{index}=[];").expect("Source buffer");
    }
    source.push_str("return isdefined(a69999);");
    source.push('}');
    let module = module_compile_support::compile_with_syntax_budget(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
        LegacySyntaxBudget {
            max_nodes: 4_000_000,
            max_child_edges: 5_000_000,
            max_metadata_bytes: 512 * 1024 * 1024,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Wide independent source arrays");
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 70_000,
            max_object_slots: 70_000,
            max_array_values: 70_001,
            max_array_value_slots: 70_001,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Wide independently configured stores");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Wide constructors");
    assert_eq!(result.value, number(1));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.live_arrays,
            usage.live_fields,
            usage.live_array_values
        ),
        (70_000, 0, 0)
    );
    assert_eq!(
        (usage.object_slots, usage.array_value_slots),
        (70_000, 70_001)
    );
    drop(context);
    assert_eq!(
        objects
            .reclaim(&[], work)
            .expect("No retained owner history")
            .objects,
        70_000
    );
}

#[test]
fn one_hundred_thousand_empty_constructions_reuse_one_value_slot_and_two_data_slots() {
    let module = module_compile_support::compile(
        b"main(){i=0;while(i<100000){a=[];a=undefined;i++;}return 1;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Construction churn");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 2,
            max_object_slots: 2,
            max_array_values: 1,
            max_array_value_slots: 1,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Bounded retained storage");
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_steps: 16_000_000,
            max_scratch_bytes: 4096,
        },
    )
    .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget {
                max_steps: 3_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Bounded churn");
    assert_eq!(result.value, number(1));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.array_value_slots,
            usage.live_array_values,
            usage.object_slots
        ),
        (1, 0, 2)
    );
    assert_eq!(usage.live_fields, 0);
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (49_999, 49_999)
    );
    assert!(context.usage().peak_scratch_bytes <= 4096);
}
