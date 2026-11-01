// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests use valid localized handles and deliberate proof/lifetime failures."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelBranchCondition, KernelBuiltin, KernelCallBudget, KernelErrorKind, KernelExecutionBudget,
    KernelFunctionLayout, KernelHeapContext, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::{LegacyComparison, LegacyNumber};

#[test]
fn localized_handles_copy_through_calls_and_locals_without_copying_payload_or_losing_the_tag() {
    let root = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::FunctionReference(1),
        KernelInstruction::CallIndirect { arguments: 1 },
        KernelInstruction::Return,
    ];
    let copy = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Duplicate,
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[definition(&root, 1, 1), definition(&copy, 1, 2)]);
    let mut heap = kernel_heap_support::heap(1);
    let id = allocate(&mut heap, b"PrivateLocalized\0\xff");
    let before = heap.usage();
    assert_eq!(
        module
            .execute(
                0,
                &[KernelValue::LocalizedString(id)],
                KernelCallBudget::default()
            )
            .expect_err("Explicit heap required")
            .cause
            .kind,
        KernelErrorKind::HeapContextRequired
    );
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::LocalizedString(id)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Copied localized handle");
    assert_eq!(result.value, KernelValue::LocalizedString(id));
    assert_ne!(result.value, KernelValue::String(id));
    assert_eq!(context.heap_usage(), before);
    assert_eq!(
        context.get(id).expect("Exact bytes"),
        b"PrivateLocalized\0\xff"
    );
    assert_eq!(
        format!("{:?}", result.value),
        "KernelValue::LocalizedString(..)"
    );
}

#[test]
fn same_literal_row_can_keep_plain_and_localized_tags_with_one_cached_payload() -> Result<(), String>
{
    let literals = vec![b"MENU_KEY".to_vec()];
    let code = [
        KernelInstruction::LocalizedStringLiteral(0),
        KernelInstruction::StoreLocal(0),
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Builtin {
            function: KernelBuiltin::IsString,
            arguments: 1,
        },
        KernelInstruction::StoreLocal(1),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfFalse,
            target: 9,
        },
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Return,
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::new(0, 2).expect("Layout"),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Separate typed loads");
    let mut heap = kernel_heap_support::heap(1);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
        .expect("Shared literal row");
    let KernelValue::LocalizedString(id) = result.value else {
        return Err("Localized tag lost".into());
    };
    assert_eq!(context.get(id).expect("Cached bytes"), b"MENU_KEY");
    assert_eq!(context.heap_usage().live_strings, 1);
    assert_eq!(context.heap_usage().payload_bytes, 8);
    assert_eq!(context.usage().collections, 0);
    Ok(())
}

#[test]
fn localized_language_equality_uses_contents_instead_of_uncanonicalized_owned_handle_identity() {
    let program = kernel_heap_support::binary(KernelInstruction::Compare(LegacyComparison::Equal));
    let mut heap = kernel_heap_support::heap(2);
    let left = allocate(&mut heap, b"MENU_KEY");
    let right = allocate(&mut heap, b"MENU_KEY");
    assert_ne!(left, right);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    assert_eq!(
        program
            .execute_with_heap(
                &[
                    KernelValue::LocalizedString(left),
                    KernelValue::LocalizedString(right)
                ],
                KernelExecutionBudget::default(),
                &mut context
            )
            .expect("Canonical content equality")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
}

#[test]
fn localized_operands_and_runtime_constants_are_checked_even_when_unreachable() {
    let mut heap = kernel_heap_support::heap(1);
    let id = allocate(&mut heap, b"private");
    let code = [
        KernelInstruction::Jump(2),
        KernelInstruction::Constant(KernelValue::LocalizedString(id)),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect_err("No runtime code handles")
            .kind,
        KernelErrorKind::RuntimeStringConstant
    );
    for index in [0, usize::MAX] {
        let code = [
            KernelInstruction::Jump(2),
            KernelInstruction::LocalizedStringLiteral(index),
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::Return,
        ];
        let error = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect_err("Missing table/unreachable index");
        assert_eq!(error.instruction, Some(1));
        assert_eq!(
            error.kind,
            KernelErrorKind::InvalidStringLiteral {
                literal: index as u64
            }
        );
    }
}

#[test]
fn localized_stale_and_foreign_arguments_fail_before_execution() {
    let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Identity");
    let mut heap = kernel_heap_support::heap(1);
    let stale = allocate(&mut heap, b"old");
    heap.reclaim(&[], HeapWorkBudget::default())
        .expect("Remove unrooted handle");
    let mut other = StringHeap::new(StringHeapBudget::default()).expect("Other owner");
    let foreign = allocate(&mut other, b"foreign");
    for (id, kind) in [
        (stale, KernelErrorKind::StaleStringReference),
        (foreign, KernelErrorKind::ForeignStringReference),
    ] {
        assert_eq!(
            program
                .execute_with_arguments(
                    &[KernelValue::LocalizedString(id)],
                    KernelExecutionBudget::default()
                )
                .expect_err("No implicit heap")
                .kind,
            KernelErrorKind::HeapContextRequired
        );
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        let error = program
            .execute_with_heap(
                &[KernelValue::LocalizedString(id)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect_err("Validate localized handle");
        assert_eq!(error.kind, kind);
        assert_eq!(error.work_steps, 0);
        assert_eq!(context.usage().steps, 1);
    }
}
