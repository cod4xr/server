// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise exact admitted string access resources."
)]
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelResource, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::{LegacyArithmetic, LegacyNumber};

#[test]
fn index_work_is_prepaid_and_one_less_never_publishes_a_partial_string() -> Result<(), String> {
    let program = kernel_heap_support::binary(KernelInstruction::Index);
    for maximum in 0..=7 {
        let mut heap = kernel_heap_support::heap(2);
        let id = allocate(&mut heap, b"xy");
        let before = heap.usage();
        let mut context = KernelHeapContext::new(
            &mut heap,
            &[],
            HeapWorkBudget {
                max_steps: maximum,
                ..HeapWorkBudget::default()
            },
        );
        let result = program.execute_with_heap(
            &[
                KernelValue::Number(LegacyNumber::Integer(1)),
                KernelValue::String(id),
            ],
            KernelExecutionBudget::default(),
            &mut context,
        );
        if maximum == 7 {
            let KernelValue::String(result) = result.expect("Exact work").value else {
                return Err("Expected string result".into());
            };
            assert_eq!(context.get(result).expect("Byte"), b"y");
            assert_eq!(context.usage().steps, 7);
            assert_eq!(context.heap_usage().payload_bytes, 3);
        } else {
            assert!(matches!(result.expect_err("Insufficient work").kind,
                KernelErrorKind::BudgetExceeded { resource: KernelResource::HeapWorkSteps, maximum: limit, .. } if limit == maximum));
            assert_eq!(context.heap_usage(), before);
            assert!(context.usage().steps <= maximum);
        }
        assert_eq!(context.get(id).expect("Unchanged original"), b"xy");
    }
    Ok(())
}

#[test]
fn size_scan_has_exact_work_and_no_heap_allocation() {
    let program = VerifiedKernelProgram::verify_function(
        &[
            KernelInstruction::LoadLocal(0),
            KernelInstruction::Size,
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Unary proof");
    for maximum in [5, 6] {
        let mut heap = kernel_heap_support::heap(1);
        let id = allocate(&mut heap, b"a\0bc");
        let before = heap.usage();
        let mut context = KernelHeapContext::new(
            &mut heap,
            &[],
            HeapWorkBudget {
                max_steps: maximum,
                ..HeapWorkBudget::default()
            },
        );
        let result = program.execute_with_heap(
            &[KernelValue::String(id)],
            KernelExecutionBudget::default(),
            &mut context,
        );
        if maximum == 6 {
            assert_eq!(
                result.expect("Conservative complete scan").value,
                KernelValue::Number(LegacyNumber::Integer(1))
            );
            assert_eq!(context.usage().steps, 6);
        } else {
            assert_eq!(
                result.expect_err("One less").kind,
                KernelErrorKind::BudgetExceeded {
                    resource: KernelResource::HeapWorkSteps,
                    requested: 6,
                    maximum: 5,
                }
            );
            assert_eq!(context.usage().steps, 2);
        }
        assert_eq!(context.heap_usage(), before);
    }
    assert_eq!(
        VerifiedKernelProgram::verify(
            &[KernelInstruction::Size, KernelInstruction::Return],
            KernelVerificationBudget::default()
        )
        .expect_err("Missing operand")
        .kind,
        KernelErrorKind::StackUnderflow {
            required: 1,
            available: 0
        }
    );
}

#[test]
fn pressure_collection_preserves_temporary_string_object_until_index_result_publication()
-> Result<(), String> {
    let code = [
        KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Integer(2))),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Index,
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(2, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Temporary object");
    let mut heap = kernel_heap_support::heap(4);
    let left = allocate(&mut heap, b"ab");
    let right = allocate(&mut heap, b"cd");
    let garbage = allocate(&mut heap, b"garbage");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = program
        .execute_with_heap(
            &[KernelValue::String(left), KernelValue::String(right)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect("GC during byte allocation");
    let KernelValue::String(id) = result.value else {
        return Err("Missing indexed byte".into());
    };
    assert_eq!(context.get(id).expect("Returned byte"), b"c");
    assert_eq!(context.get(left).expect("Left local"), b"ab");
    assert_eq!(context.get(right).expect("Right local"), b"cd");
    assert!(context.get(garbage).is_err());
    assert_eq!(context.usage().collections, 1);
    assert_eq!(context.heap_usage().slots, 4);
    assert_eq!(context.heap_usage().payload_bytes, 9);
    Ok(())
}
