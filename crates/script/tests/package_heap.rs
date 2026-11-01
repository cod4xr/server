// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute caller/callee object and string roots through shared heap pressure."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::ScriptPackageBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn cross_file_frames_and_literal_caches_remain_rooted_under_shared_heap_pressure() {
    let package = compile(&[
        (b"root", b"main(){r=spawnstruct();r.text=\"\";r.anchor=&\"MENU\";i=0;while(i<1000){lib::append(r);i++;}return r.text.size+isdefined(r.anchor);}"),
        (b"lib", b"append(r){r.text+=\"x\";r.temp=spawnstruct();}"),
    ], Disabled, ScriptPackageBudget::default()).expect("Owned package");
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 5,
        max_slots: 5,
        max_payload_bytes: 4096,
        ..StringHeapBudget::default()
    })
    .expect("Five string slots");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 3,
            max_object_slots: 3,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Three object slots");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let outcome = package
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Caller roots survive callee pressure");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(1001))
    );
    assert!(context.usage().collections > 900);
    assert!(context.usage().object_collections > 900);
    assert_eq!(context.heap_usage().slots, 5);
    assert_eq!(
        context.object_heap_usage().expect("Objects").object_slots,
        3
    );
    let exact = context.usage();
    drop(context);
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_steps: exact.steps - 1,
            ..HeapWorkBudget::default()
        },
    )
    .with_objects(&mut objects, &[]);
    let error = package
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect_err("A second invocation has one shared heap budget");
    assert!(context.usage().steps < exact.steps);
    let location = package
        .instruction_location(
            error.function.expect("Function"),
            error.cause.instruction.expect("Instruction"),
        )
        .expect("Private location");
    assert!(location.module <= 1);
}
