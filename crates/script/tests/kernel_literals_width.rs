// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute wide immutable tables and repeated large cached literals."
)]
use crate::kernel_call_support::integer;
use crate::kernel_heap_support;
use cod4x_script::execution::{
    KernelBranchCondition, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::{LegacyComparison, LegacyIntegerUpdate};

#[test]
fn immutable_literal_tables_and_invocation_cache_indices_exceed_native_pool_and_short_widths()
-> Result<(), String> {
    let literals: Vec<_> = (0..70_000_u32)
        .map(|index| index.to_le_bytes().to_vec())
        .collect();
    let code = [
        KernelInstruction::StringLiteral(65_535),
        KernelInstruction::Discard,
        KernelInstruction::StringLiteral(65_536),
        KernelInstruction::Discard,
        KernelInstruction::StringLiteral(69_999),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::default(),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Wide complete literal proof");
    assert_eq!(program.usage().string_literals, 70_000);
    assert_eq!(
        program.usage().retained_string_bytes,
        70_000 * (size_of::<Vec<u8>>() as u64 + 4)
    );
    let mut heap = kernel_heap_support::heap(3);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
        .expect("Wide cache operands");
    let KernelValue::String(id) = result.value else {
        return Err("Expected wide literal".into());
    };
    assert_eq!(
        context.get(id).expect("No truncated index"),
        69_999_u32.to_le_bytes()
    );
    assert_eq!(context.heap_usage().live_strings, 3);
    assert_eq!(context.heap_usage().payload_bytes, 12);
    assert_eq!(context.usage().steps, 70_021);
    assert!(context.usage().peak_scratch_bytes > 1_000_000);
    Ok(())
}

#[test]
fn repeated_large_literal_evaluation_copies_payload_once_per_invocation() -> Result<(), String> {
    let literals = vec![vec![0xff; 1024 * 1024]];
    let code = [
        integer(0),
        KernelInstruction::StoreLocal(0),
        KernelInstruction::LoadLocal(0),
        integer(1000),
        KernelInstruction::Compare(LegacyComparison::Less),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfFalse,
            target: 12,
        },
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::IntegerUpdate(LegacyIntegerUpdate::Increment),
        KernelInstruction::StoreLocal(0),
        KernelInstruction::Jump(2),
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::new(0, 1).expect("Counter layout"),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Repeated literal proof");
    let mut heap = kernel_heap_support::heap(1);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
        .expect("Cached large value");
    let KernelValue::String(id) = result.value else {
        return Err("Expected large literal".into());
    };
    assert_eq!(context.get(id).expect("Exact bytes"), literals[0]);
    assert_eq!(context.heap_usage().live_strings, 1);
    assert_eq!(context.heap_usage().payload_bytes, 1024 * 1024);
    assert_eq!(context.usage().collections, 0);
    assert_eq!(context.usage().steps, 1024 * 1024 + 2004);
    assert_eq!(result.steps, 10_008);
    Ok(())
}
