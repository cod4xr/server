// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile source constructors, shadowing and real allocation pressure."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectResource, StringHeap,
    StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn source_constructor_shadowing_queries_and_evaluated_extra_failures_preserve_original_locations() {
    for (source, expected_objects, expected) in [
        (
            b"main(){value=SpAwNsTrUcT(1,(2,3,4));return value.size+isdefined(value);}".as_slice(),
            1,
            2,
        ),
        (
            b"main(){return spawnstruct();}spawnstruct(){return 7;}",
            0,
            7,
        ),
    ] {
        let module = module_compile_support::compile(
            source,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Source constructors and script shadowing");
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        assert_eq!(
            module
                .program()
                .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
                .expect("Source execution")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Object backing")
                .live_objects,
            expected_objects
        );
    }
    let source = b"main(){return spawnstruct(1/0);}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Argument error code");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let error = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect_err("Arguments execute before allocation");
    let span = module
        .instruction_span(
            error.function.expect("Function"),
            error.cause.instruction.expect("Instruction"),
        )
        .expect("Original operator");
    assert_eq!(
        &source
            [usize::try_from(span.start).expect("Start")..usize::try_from(span.end).expect("End")],
        b"/"
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Not allocated")
            .live_objects,
        0
    );
}

#[test]
fn source_constructor_extras_stay_rooted_until_outer_allocation_and_cannot_evade_live_budget() {
    let source = b"main(){return spawnstruct(spawnstruct(),spawnstruct());}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Nested constructors");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 2,
            max_object_slots: 2,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let error = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect_err("Both extra operands remain roots");
    assert!(matches!(
        error.cause.kind,
        cod4x_script::execution::KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::LiveObjects,
            requested: 3,
            maximum: 2
        })
    ));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Operands preserved")
            .live_objects,
        2
    );
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (1, 1)
    );
}

#[test]
fn source_constructor_churn_runs_100000_iterations_with_two_object_slots_and_global_work() {
    let source =
        b"main(){value=spawnstruct();for(i=0;i<100000;i++){value=spawnstruct();}return value;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Constructor churn loop");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 2,
            max_object_slots: 2,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget {
                max_steps: 2_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Bounded real source churn");
    assert!(matches!(result.value, KernelValue::Object(_)));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Reused slots")
            .object_slots,
        2
    );
    assert_eq!(context.usage().object_collections, 99_999);
    assert_eq!(context.usage().collections, 99_999);
    assert!(context.usage().steps < HeapWorkBudget::default().max_steps);
}
