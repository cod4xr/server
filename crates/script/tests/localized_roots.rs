// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests isolate cached/caller localized roots under real allocation pressure."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelInstruction, KernelValue,
};
use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget};
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn collection_keeps_localized_caller_stack_cached_only_and_external_roots() -> Result<(), String> {
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
    let worker = [
        KernelInstruction::LocalizedStringLiteral(0),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(0),
        clear,
        KernelInstruction::StoreLocal(0),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let literals = vec![b"cached localized".to_vec()];
    let mut definitions = [definition(&root, 2, 2), definition(&worker, 1, 1)];
    definitions[1].string_literals = &literals;
    let module = kernel_call_support::verify(&definitions);
    let mut heap = kernel_heap_support::heap(5);
    let held = allocate(&mut heap, b"caller localized");
    let operand = allocate(&mut heap, b"x");
    let external = allocate(&mut heap, b"external localized");
    let garbage = allocate(&mut heap, b"garbage");
    let roots = [external];
    let mut context = KernelHeapContext::new(&mut heap, &roots, HeapWorkBudget::default());
    let result = module
        .execute_with_heap(
            0,
            &[
                KernelValue::LocalizedString(held),
                KernelValue::String(operand),
            ],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Mixed string-category roots");
    assert_eq!(result.value, KernelValue::LocalizedString(held));
    assert_eq!(
        context.get(held).expect("Caller stack root"),
        b"caller localized"
    );
    assert_eq!(
        context.get(external).expect("External root"),
        b"external localized"
    );
    assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
    assert_eq!(context.usage().collections, 1);
    assert_eq!(context.heap_usage().live_strings, 5);
    let result = module
        .execute_with_heap(
            1,
            &[KernelValue::String(operand)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Recreated invocation cache");
    let KernelValue::String(id) = result.value else {
        return Err("Expected plain worker result".into());
    };
    assert_eq!(context.get(id).expect("Surviving plain operands"), b"xx");
    assert_eq!(
        context.get(external).expect("External localized root"),
        b"external localized"
    );
    Ok(())
}
