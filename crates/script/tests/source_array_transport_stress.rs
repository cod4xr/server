// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure wide managed VM owners and bounded overwrite churn beside unrelated host aliases."
)]
use std::fmt::Write;

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::syntax::LegacySyntaxBudget;

#[test]
fn seventy_thousand_owned_source_locals_release_without_short_owner_indices() {
    let mut source = String::from("main(a){");
    for index in 0..70_000 {
        write!(source, "p{index}=a;").expect("Source buffer");
    }
    source.push_str("return isdefined(p69999);}");
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
    .expect("Wide local source");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_values: 70_003,
            max_array_value_slots: 70_003,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Wide managed arena");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let data = objects
        .array_data(host, HeapWorkBudget::default())
        .expect("Data")
        .value;
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Wide owned locals");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (usage.array_value_slots, usage.live_array_values),
        (70_003, 1)
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Owners")
            .value,
        1
    );
}

#[test]
fn overwritten_aliases_reuse_slots_with_small_scratch_beside_seventy_thousand_host_owners() {
    let mut source = String::from("main(a){");
    for index in 0..32 {
        write!(source, "p{index}=a;").expect("Source buffer");
    }
    source.push_str("b=undefined;i=0;while(i<100000){b=a;i++;}return isdefined(b);}");
    let module = module_compile_support::compile(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Overwrite loop");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_values: 70_036,
            max_array_value_slots: 70_036,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Bounded managed arena");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let data = objects
        .array_data(host, HeapWorkBudget::default())
        .expect("Data")
        .value;
    for _ in 0..70_000 {
        objects
            .copy_array_value(host, HeapWorkBudget::default())
            .expect("Unrelated host alias");
    }
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
            &[KernelValue::Array(host)],
            KernelCallBudget {
                max_steps: 4_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Bounded alias churn");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (usage.array_value_slots, usage.live_array_values),
        (70_036, 70_001)
    );
    assert!(context.usage().peak_scratch_bytes <= 4096);
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Owners")
            .value,
        70_001
    );
}
