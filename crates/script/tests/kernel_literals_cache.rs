// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests force cached-only roots and independent immutable table namespaces."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelModuleVerificationBudget, KernelValue, KernelVerificationBudget,
    VerifiedKernelModule, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget};
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn one_immutable_program_executes_against_independent_heap_owners() -> Result<(), String> {
    let literals = vec![b"shared code".to_vec()];
    let program = VerifiedKernelProgram::verify_with_literals(
        &[
            KernelInstruction::StringLiteral(0),
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::default(),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Independent immutable code");
    let mut first_heap = kernel_heap_support::heap(1);
    let mut second_heap = kernel_heap_support::heap(1);
    let mut first = KernelHeapContext::new(&mut first_heap, &[], HeapWorkBudget::default());
    let mut second = KernelHeapContext::new(&mut second_heap, &[], HeapWorkBudget::default());
    let result = program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut first)
        .expect("First owner");
    let KernelValue::String(left) = result.value else {
        return Err("Expected first string".into());
    };
    let result = program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut second)
        .expect("Second owner");
    let KernelValue::String(right) = result.value else {
        return Err("Expected second string".into());
    };
    assert_ne!(left, right);
    assert_eq!(
        first.get(left).expect("First bytes"),
        second.get(right).expect("Second bytes")
    );
    assert_eq!(first.get(right), Err(HeapErrorKind::ForeignReference));
    assert_eq!(second.get(left), Err(HeapErrorKind::ForeignReference));
    Ok(())
}

#[test]
fn pressure_collection_preserves_literal_references_held_only_by_the_cache() -> Result<(), String> {
    let literals = vec![b"cached only".to_vec()];
    let code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Discard,
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Cached-only root program");
    let mut heap = kernel_heap_support::heap(3);
    let argument = allocate(&mut heap, b"x");
    let garbage = allocate(&mut heap, b"old");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = program
        .execute_with_heap(
            &[KernelValue::String(argument)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect("Cached root survives");
    let KernelValue::String(id) = result.value else {
        return Err("Expected cached string".into());
    };
    assert_eq!(context.get(id).expect("Cached bytes"), b"cached only");
    assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
    assert_eq!(context.usage().collections, 1);
    assert_eq!(context.heap_usage().live_strings, 3);
    Ok(())
}

#[test]
fn module_function_tables_have_distinct_offsets_and_one_global_invocation_cache()
-> Result<(), String> {
    let root = [
        KernelInstruction::Call {
            function: 1,
            arguments: 0,
        },
        KernelInstruction::Call {
            function: 2,
            arguments: 0,
        },
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let literal = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Return,
    ];
    let first = vec![b"foo".to_vec()];
    let second = vec![b"bar".to_vec()];
    let mut definitions = [
        definition(&root, 0, 0),
        definition(&literal, 0, 0),
        definition(&literal, 0, 0),
    ];
    definitions[1].string_literals = &first;
    definitions[2].string_literals = &second;
    let module = kernel_call_support::verify(&definitions);
    assert_eq!(module.usage().string_literals, 2);
    let mut heap = kernel_heap_support::heap(3);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Distinct local row zero");
    let KernelValue::String(id) = result.value else {
        return Err("Expected concat".into());
    };
    assert_eq!(context.get(id).expect("Module literals"), b"foobar");
    Ok(())
}

#[test]
fn reusing_context_for_different_modules_discards_old_cache_and_preserves_global_work()
-> Result<(), String> {
    let code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Return,
    ];
    let first = vec![b"first".to_vec()];
    let second = vec![b"second".to_vec()];
    let mut input = definition(&code, 0, 0);
    input.string_literals = &first;
    let first = VerifiedKernelModule::verify(&[input], KernelModuleVerificationBudget::default())
        .expect("First module");
    input.string_literals = &second;
    let second = VerifiedKernelModule::verify(&[input], KernelModuleVerificationBudget::default())
        .expect("Second module");
    let mut heap = kernel_heap_support::heap(1);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = first
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("First literal");
    let KernelValue::String(old) = result.value else {
        return Err("Expected first string".into());
    };
    let work = context.usage().steps;
    let result = second
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("New invocation namespace");
    let KernelValue::String(new) = result.value else {
        return Err("Expected second string".into());
    };
    assert_eq!(context.get(new).expect("New module bytes"), b"second");
    assert_eq!(context.get(old), Err(HeapErrorKind::StaleReference));
    assert_eq!(context.usage().collections, 1);
    assert!(context.usage().steps > work);
    Ok(())
}
