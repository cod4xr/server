// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile and execute deep writable field chains without native recursion or short offsets."
)]
use std::fmt::Write;

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationResource, FunctionCompileErrorKind, ModuleCompilationBudget,
};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn twelve_thousand_nested_field_receivers_lower_update_execute_and_fail_cleanup_iteratively() {
    let mut source = String::from(
        "main(){root=spawnstruct();cursor=root;i=0;while(i<12000){cursor.child=spawnstruct();cursor=cursor.child;i++;}cursor.value=1;",
    );
    let target = format!("root{}.value", ".child".repeat(12_000));
    write!(
        source,
        "{target}+=2;{target}++;{target}--;{target}++;return {target};}}"
    )
    .expect("Source buffer");
    let module = module_compile_support::compile(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Deep field update tasks");
    assert!(module.usage().instructions > 65_535);
    source.clear();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Deep read/compound/postfix chains")
            .value,
        KernelValue::Number(LegacyNumber::Integer(4))
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Graph fields")
            .live_fields,
        12_001
    );
    assert_eq!(context.heap_usage().live_strings, 0);
    let source = format!("main(root){{{target}++;}}");
    let mut budget = ModuleCompilationBudget::default();
    budget.function.max_task_frames = 6000;
    let error =
        module_compile_support::compile(source.as_bytes(), LegacyDeveloperMode::Disabled, budget)
            .expect_err("Failure partway through receiver lowering");
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::TaskFrames,
            requested: 6001,
            maximum: 6000
        }
    ));
}
