// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute valid owned binary strings and deliberate type errors."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction,
    KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::{LegacyArithmetic, LegacyComparison, LegacyNumber};

#[test]
fn opaque_strings_copy_through_locals_direct_and_indirect_calls_without_copying_payload() {
    let root = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::FunctionReference(1),
        KernelInstruction::CallIndirect { arguments: 1 },
        KernelInstruction::Call {
            function: 2,
            arguments: 1,
        },
        KernelInstruction::Return,
    ];
    let identity = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Duplicate,
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[
        definition(&root, 1, 1),
        definition(&identity, 1, 2),
        definition(&identity, 1, 2),
    ]);
    let mut heap = kernel_heap_support::heap(1);
    let bytes = vec![0xff; 200_000];
    let id = allocate(&mut heap, &bytes);
    let before = heap.usage();
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::String(id)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Copy through calls");
    assert_eq!(result.value, KernelValue::String(id));
    assert_eq!(context.get(id).expect("Retained result"), bytes);
    assert_eq!(context.heap_usage(), before);
    assert_eq!(context.usage().steps, 1);
    assert_eq!(context.usage().collections, 0);
    assert_eq!(format!("{:?}", result.value), "KernelValue::String(..)");
    assert!(!format!("{context:?}").contains("StringId"));
}

#[test]
fn string_language_equality_uses_bytes_and_unsupported_coercion_and_ordering_fail() {
    let mut heap = kernel_heap_support::heap(8);
    let left = allocate(&mut heap, b"secret\0\xff");
    let right = allocate(&mut heap, b"secret\0\xff");
    let different = allocate(&mut heap, b"secret\0\xfe");
    assert_ne!(left, right);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    for comparison in [LegacyComparison::Equal, LegacyComparison::NotEqual] {
        let program = kernel_heap_support::binary(KernelInstruction::Compare(comparison));
        for (other, equal) in [(right, true), (different, false), (left, true)] {
            let result = program
                .execute_with_heap(
                    &[KernelValue::String(left), KernelValue::String(other)],
                    KernelExecutionBudget::default(),
                    &mut context,
                )
                .expect("Byte comparison");
            let expected = equal == (comparison == LegacyComparison::Equal);
            assert_eq!(
                result.value,
                KernelValue::Number(LegacyNumber::Integer(i32::from(expected)))
            );
        }
    }
    for instruction in [
        KernelInstruction::Compare(LegacyComparison::Less),
        KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
        KernelInstruction::Divide,
    ] {
        let program = kernel_heap_support::binary(instruction);
        assert_eq!(
            program
                .execute_with_heap(
                    &[KernelValue::String(left), KernelValue::String(right)],
                    KernelExecutionBudget::default(),
                    &mut context
                )
                .expect_err("Unsupported operator")
                .kind,
            KernelErrorKind::UnmatchingTypes
        );
    }
    let add = kernel_heap_support::binary(KernelInstruction::Arithmetic(LegacyArithmetic::Add));
    for arguments in [
        [KernelValue::String(left), KernelValue::Undefined],
        [
            KernelValue::Number(LegacyNumber::Integer(3)),
            KernelValue::String(right),
        ],
    ] {
        assert_eq!(
            add.execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context)
                .expect_err("No guessed coercion")
                .kind,
            KernelErrorKind::UnmatchingTypes
        );
    }
}

#[test]
fn runtime_heap_references_are_rejected_as_constants_including_unreachable_code() {
    let mut heap = kernel_heap_support::heap(1);
    let id = allocate(&mut heap, b"not a code root");
    let code = [
        KernelInstruction::Jump(2),
        KernelInstruction::Constant(KernelValue::String(id)),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let error = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect_err("Unreachable constant checked");
    assert_eq!(error.instruction, Some(1));
    assert_eq!(error.kind, KernelErrorKind::RuntimeStringConstant);
}
