// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests provide complete call inventories with intentional dynamic argument errors."
)]

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelBranchCondition, KernelCallBudget, KernelErrorKind, KernelInstruction,
    KernelModuleVerificationBudget, KernelValue, VerifiedKernelModule,
};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};

#[test]
fn skipped_excess_calls_compile_but_selected_calls_fail_at_the_caller_location() {
    let root = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfFalse,
            target: 5,
        },
        integer(17379),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Return,
        integer(9),
        KernelInstruction::Return,
    ];
    let leaf = [integer(1), KernelInstruction::Return];
    let module = kernel_call_support::verify(&[definition(&root, 1, 1), definition(&leaf, 0, 2)]);
    let budget = KernelCallBudget {
        max_frames: 1,
        ..KernelCallBudget::default()
    };
    assert_eq!(
        module
            .execute(0, &[KernelValue::Number(LegacyNumber::Integer(0))], budget)
            .expect("Unselected excess call is harmless")
            .value,
        KernelValue::Number(LegacyNumber::Integer(9))
    );
    let error = module
        .execute(0, &[KernelValue::Number(LegacyNumber::Integer(1))], budget)
        .expect_err("Selected call fails even though extra argument fits target local storage");
    assert_eq!(error.function, Some(0));
    assert_eq!(error.cause.instruction, Some(3));
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 0,
            actual: 1
        }
    );
    assert_eq!(error.cause.work_steps, 4);
    assert_eq!(error.peak_frames, 1);
    assert_eq!(error.setup_work, 3);
    assert!(!format!("{error:?} {error}").contains("17379"));
    assert_eq!(
        module
            .execute(1, &[], budget)
            .expect("Fresh entry still works after call failure")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
}

#[test]
fn caller_argument_failure_precedes_count_error_and_count_error_precedes_body_execution() {
    let bad_argument = [
        integer(1),
        integer(0),
        KernelInstruction::Divide,
        KernelInstruction::Call {
            function: 2,
            arguments: 1,
        },
        KernelInstruction::Return,
    ];
    let bad_count = [
        integer(3),
        KernelInstruction::Call {
            function: 2,
            arguments: 1,
        },
        KernelInstruction::Return,
    ];
    let bad_body = [
        integer(1),
        integer(0),
        KernelInstruction::Divide,
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[
        definition(&bad_argument, 0, 0),
        definition(&bad_count, 0, 0),
        definition(&bad_body, 0, 0),
    ]);
    let argument = module
        .execute(0, &[], KernelCallBudget::default())
        .expect_err("Caller argument runs first");
    assert_eq!(
        argument.cause.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    assert_eq!(argument.function, Some(0));
    assert_eq!(argument.cause.instruction, Some(2));
    assert_eq!(argument.cause.work_steps, 3);
    let count = module
        .execute(1, &[], KernelCallBudget::default())
        .expect_err("Bad count does not enter callee body");
    assert_eq!(
        count.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 0,
            actual: 1
        }
    );
    assert_eq!(count.function, Some(1));
    assert_eq!(count.cause.instruction, Some(1));
    assert_eq!(count.cause.work_steps, 2);
    assert_eq!(count.peak_frames, 1);
    assert_eq!(count.setup_work, 1);
    let body = module
        .execute(2, &[], KernelCallBudget::default())
        .expect_err("Body would fail when validly invoked");
    assert_eq!(
        body.cause.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    assert_eq!(body.function, Some(2));
}

#[test]
fn runtime_arity_does_not_weaken_operand_proof_for_huge_reachable_counts() {
    let root = [
        KernelInstruction::Call {
            function: 1,
            arguments: usize::MAX,
        },
        KernelInstruction::Return,
    ];
    let leaf = [integer(1), KernelInstruction::Return];
    let error = VerifiedKernelModule::verify(
        &[definition(&root, 0, 0), definition(&leaf, 0, 0)],
        KernelModuleVerificationBudget::default(),
    )
    .expect_err("Reachable operands must exist before runtime count validation");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::StackUnderflow {
            required: usize::MAX as u64,
            available: 0
        }
    );
    assert_eq!(error.function, Some(0));
    assert_eq!(error.cause.instruction, Some(0));
}
