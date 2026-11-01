// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests provide owned literal tables and explicit invalid operands/budgets."
)]
use crate::kernel_call_support::definition;
use crate::kernel_heap_support;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelModuleVerificationBudget, KernelResource, KernelValue,
    KernelVerificationBudget, VerifiedKernelModule, VerifiedKernelProgram,
};
use cod4x_script::heap::HeapWorkBudget;

#[test]
fn immutable_literals_own_exact_private_bytes_after_input_drop_and_require_explicit_runtime_heap()
-> Result<(), String> {
    let code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Return,
    ];
    let mut literals = vec![vec![0xff; 200_000]];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::default(),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Owned literal proof");
    literals[0].fill(0);
    drop(literals);
    assert_eq!(program.usage().string_literals, 1);
    assert_eq!(
        program.usage().retained_string_bytes,
        size_of::<Vec<u8>>() as u64 + 200_000
    );
    let error = program
        .execute(KernelExecutionBudget::default())
        .expect_err("No implicit dropped heap");
    assert_eq!(error.instruction, Some(0));
    assert_eq!(error.work_steps, 1);
    assert_eq!(error.kind, KernelErrorKind::HeapContextRequired);
    let mut heap = kernel_heap_support::heap(1);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
        .expect("Explicit owned literal");
    let KernelValue::String(id) = result.value else {
        return Err("Expected string literal result".into());
    };
    assert_eq!(
        context.get(id).expect("Retained bytes"),
        vec![0xff; 200_000]
    );
    assert_eq!(context.usage().steps, 200_004);
    assert_eq!(context.heap_usage().live_strings, 1);
    Ok(())
}

#[test]
fn every_literal_operand_is_checked_including_unreachable_code_and_missing_tables() {
    let literals = vec![b"PrivateLiteral".to_vec()];
    for index in [1, usize::MAX] {
        let code = [
            KernelInstruction::Jump(2),
            KernelInstruction::StringLiteral(index),
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::Return,
        ];
        let error = VerifiedKernelProgram::verify_with_literals(
            &code,
            KernelFunctionLayout::default(),
            &literals,
            KernelVerificationBudget::default(),
        )
        .expect_err("Unreachable invalid literal");
        assert_eq!(error.instruction, Some(1));
        assert_eq!(
            error.kind,
            KernelErrorKind::InvalidStringLiteral {
                literal: index as u64
            }
        );
        let mut input = definition(&code, 0, 0);
        input.string_literals = &literals;
        let error =
            VerifiedKernelModule::verify(&[input], KernelModuleVerificationBudget::default())
                .expect_err("Module literal operand");
        assert_eq!(error.function, Some(0));
        assert_eq!(
            error.cause.kind,
            KernelErrorKind::InvalidStringLiteral {
                literal: index as u64
            }
        );
        assert!(!format!("{input:?} {error:?}").contains("PrivateLiteral"));
        assert!(!format!("{input:?} {error:?}").contains(&format!("{:?}", literals[0])));
        assert!(format!("{input:?}").contains("string_literals: 1"));
    }
    let error = VerifiedKernelProgram::verify(
        &[
            KernelInstruction::StringLiteral(0),
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect_err("No fabricated table");
    assert_eq!(
        error.kind,
        KernelErrorKind::InvalidStringLiteral { literal: 0 }
    );
}

#[test]
fn unused_literal_tables_still_pay_exact_proof_work_and_retained_metadata() {
    let literals = vec![vec![7; 32]; 1000];
    let code = [
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let usage = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::default(),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Measured proof")
    .usage();
    assert_eq!(usage.string_literals, 1000);
    assert_eq!(
        usage.retained_string_bytes,
        1000 * (size_of::<Vec<u8>>() as u64 + 32)
    );
    let exact = KernelVerificationBudget {
        max_steps: usage.verification_steps,
        max_metadata_bytes: usage.verification_metadata_bytes,
        ..KernelVerificationBudget::default()
    };
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::default(),
        &literals,
        exact,
    )
    .expect("Exact admission");
    assert_eq!(
        program
            .execute(KernelExecutionBudget::default())
            .expect("Unused data creates no runtime values")
            .value,
        KernelValue::Undefined
    );
    for budget in [
        KernelVerificationBudget {
            max_steps: exact.max_steps - 1,
            ..exact
        },
        KernelVerificationBudget {
            max_metadata_bytes: exact.max_metadata_bytes - 1,
            ..exact
        },
    ] {
        let error = VerifiedKernelProgram::verify_with_literals(
            &code,
            KernelFunctionLayout::default(),
            &literals,
            budget,
        )
        .expect_err("One less proof resource");
        assert!(matches!(
            error.kind,
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::VerificationSteps
                    | KernelResource::VerificationMetadataBytes,
                ..
            }
        ));
        assert!(error.work_steps <= budget.max_steps);
    }
}
