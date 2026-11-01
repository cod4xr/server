// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests force reclamation with individually isolated live roots."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelInstruction, KernelValue,
};
use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget};
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn collection_preserves_caller_stack_callee_operands_and_external_handles_after_local_overwrites()
-> Result<(), String> {
    let clear = KernelInstruction::Constant(KernelValue::Undefined);
    let root = [
        KernelInstruction::LoadLocal(0),
        clear,
        KernelInstruction::StoreLocal(0),
        KernelInstruction::LoadLocal(1),
        clear,
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Discard,
        KernelInstruction::Return,
    ];
    let callee = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(0),
        clear,
        KernelInstruction::StoreLocal(0),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[definition(&root, 2, 2), definition(&callee, 1, 1)]);
    let mut heap = kernel_heap_support::heap(4);
    let held = allocate(&mut heap, b"caller");
    let operand = allocate(&mut heap, b"operand");
    let external = allocate(&mut heap, b"external");
    let garbage = allocate(&mut heap, b"garbage");
    let external_roots = [external];
    let mut context = KernelHeapContext::new(&mut heap, &external_roots, HeapWorkBudget::default());
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::String(held), KernelValue::String(operand)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Pressure collection");
    assert_eq!(result.value, KernelValue::String(held));
    assert_eq!(context.get(held).expect("Caller stack root"), b"caller");
    assert_eq!(
        context.get(operand).expect("Callee operand roots"),
        b"operand"
    );
    assert_eq!(context.get(external).expect("External root"), b"external");
    assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
    assert_eq!(context.usage().collections, 1);
    assert_eq!(context.heap_usage().live_strings, 4);
    let result = module
        .execute_with_heap(
            1,
            &[KernelValue::String(operand)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Returned allocated string");
    let KernelValue::String(id) = result.value else {
        return Err("Expected allocated string".into());
    };
    assert_eq!(
        context.get(id).expect("Result survives return"),
        b"operandoperand"
    );
    assert_eq!(
        context
            .get(external)
            .expect("External root survives next invocation"),
        b"external"
    );
    assert_eq!(context.get(held), Err(HeapErrorKind::StaleReference));
    Ok(())
}

#[test]
fn collection_traces_deep_iterative_call_frames_and_duplicate_local_roots() -> Result<(), String> {
    let count = 12_000;
    let mut code = Vec::new();
    for function in 0..count {
        let body = if function + 1 == count {
            vec![
                KernelInstruction::LoadLocal(0),
                KernelInstruction::LoadLocal(0),
                KernelInstruction::Arithmetic(LegacyArithmetic::Add),
                KernelInstruction::Return,
            ]
        } else {
            vec![
                KernelInstruction::LoadLocal(0),
                KernelInstruction::Call {
                    function: function + 1,
                    arguments: 1,
                },
                KernelInstruction::Return,
            ]
        };
        code.push(body);
    }
    let definitions: Vec<_> = code.iter().map(|body| definition(body, 1, 1)).collect();
    let module = kernel_call_support::verify(&definitions);
    let mut heap = kernel_heap_support::heap(2);
    let id = allocate(&mut heap, b"x");
    let garbage = allocate(&mut heap, b"old");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::String(id)],
            KernelCallBudget {
                max_setup_work: 100_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Deep call tree collection");
    let KernelValue::String(result_id) = result.value else {
        return Err("Expected string result".into());
    };
    assert_eq!(context.get(result_id).expect("Result"), b"xx");
    assert_eq!(context.get(id).expect("All caller locals"), b"x");
    assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
    assert_eq!(result.usage.peak_frames, count as u64);
    assert_eq!(context.usage().collections, 1);
    assert!(context.usage().steps > count as u64 * 3);
    Ok(())
}
