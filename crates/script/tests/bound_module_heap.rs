// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile real source and supply admitted host-owned string arguments."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn bound_source_locals_calls_and_loops_operate_on_supplied_strings_under_pressure()
-> Result<(), String> {
    let mut initialized = b"main(a,b){value=a;for(i=0;i<1000;i++){value=combine(a,b);}return value;}combine(a,b){return a+b;}".to_vec();
    let module = module_compile_support::compile(
        &initialized,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Real source operations");
    initialized.fill(0);
    let mut heap = StringHeap::new(StringHeapBudget {
        max_live_strings: 4,
        max_slots: 4,
        ..StringHeapBudget::default()
    })
    .expect("Heap");
    let left = heap
        .allocate(b"left", HeapWorkBudget::default())
        .expect("Left")
        .id;
    let right = heap
        .allocate(b"right", HeapWorkBudget::default())
        .expect("Right")
        .id;
    let arguments = [KernelValue::String(left), KernelValue::String(right)];
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &arguments, KernelCallBudget::default(), &mut context)
        .expect("Source loop strings");
    let KernelValue::String(id) = result.value else {
        return Err("Expected string".into());
    };
    assert_eq!(context.get(id).expect("Returned bytes"), b"leftright");
    assert!(context.usage().collections > 900);
    assert_eq!(context.heap_usage().slots, 4);
    assert!(!format!("{module:?} {result:?}").contains("leftright"));
    Ok(())
}

#[test]
fn heap_operation_failures_preserve_bound_source_operator_locations_and_privacy() {
    let source = b"main(a,b){return hidden(a,b);}hidden(a,b){return a+b;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Bound source maps");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let id = heap
        .allocate(b"PrivatePayload", HeapWorkBudget::default())
        .expect("Input")
        .id;
    let mut context = KernelHeapContext::new(
        &mut heap,
        &[],
        HeapWorkBudget {
            max_steps: 3,
            ..HeapWorkBudget::default()
        },
    );
    let error = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::String(id), KernelValue::String(id)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("Shared heap work failure");
    let span = module
        .instruction_span(
            error.function.expect("Function"),
            error.cause.instruction.expect("Operator"),
        )
        .expect("Original span");
    assert_eq!(
        &source[usize::try_from(span.start).expect("Offset")
            ..usize::try_from(span.end).expect("Offset")],
        b"+"
    );
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded { .. }
    ));
    assert!(!format!("{error:?} {context:?}").contains("PrivatePayload"));
    assert!(!format!("{error:?}").contains("hidden"));
}
