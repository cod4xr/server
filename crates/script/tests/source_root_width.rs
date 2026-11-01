// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile and execute wide root-field tables without inventing implicit local/world slots."
)]
use std::fmt::Write;

use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::syntax::LegacySyntaxBudget;

#[test]
fn seventy_thousand_root_fields_use_wide_owned_names_and_no_implicit_locals() {
    let mut source = String::from("main(){");
    for index in 0..70_000 {
        write!(source, "level.f{index}={index};").expect("Source");
    }
    source.push_str("return level.f69999;}");
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
    .expect("Wide root fields");
    source.clear();
    let mut state = script_root_support::state(ObjectHeapBudget::default());
    let roots = state.roots();
    let mut context = KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
        .with_objects(&mut state.objects, &[])
        .with_script_roots(roots);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Wide root writes");
    assert_eq!(
        result.value,
        KernelValue::Number(LegacyNumber::Integer(69_999))
    );
    assert_eq!(result.usage.peak_local_values, 0);
    assert_eq!(
        context.object_heap_usage().expect("Objects").live_fields,
        70_000
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    assert_eq!(context.usage().object_collections, 0);
    drop(context);
    assert_eq!(state.objects.field_count(state.level), Ok(70_000));
    assert_eq!(state.objects.field_count(state.receiver), Ok(0));
    assert_eq!(state.objects.field_count(state.animation), Ok(0));
}

#[test]
fn a_hundred_thousand_game_reads_reuse_two_locations_with_bounded_registry_scratch() {
    let module = module_compile_support::compile(
        b"main(){i=0;while(i<100000){isdefined(game);i++;}return level.size;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Root churn source");
    let mut state = script_root_support::state(ObjectHeapBudget {
        max_array_values: 2,
        max_array_value_slots: 2,
        ..ObjectHeapBudget::default()
    });
    let roots = state.roots();
    let mut context = KernelHeapContext::new(
        &mut state.strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 1024,
            ..HeapWorkBudget::default()
        },
    )
    .with_objects(&mut state.objects, &[])
    .with_script_roots(roots);
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
        .expect("Bounded game-root reads");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
    let objects = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (objects.array_value_slots, objects.live_array_values),
        (2, 1)
    );
    assert!(context.usage().peak_scratch_bytes <= 1024);
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (0, 0)
    );
}
