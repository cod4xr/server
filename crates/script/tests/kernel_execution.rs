// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual pure scalar execution, ownership, paths, and charged failures.
#![allow(
    clippy::expect_used,
    reason = "Tests assemble bounded synthetic scalar programs."
)]

use std::error::Error;

use cod4x_script::execution::{
    KernelBranchCondition, KernelErrorKind, KernelExecutionBudget, KernelInstruction,
    KernelResource, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};

use crate::kernel_support;

#[test]
fn owned_programs_retain_constants_and_redact_instruction_outcome_and_program_debug() {
    let private_value = 1_357_913_579;
    let mut input = vec![
        kernel_support::integer(private_value),
        KernelInstruction::Return,
    ];
    let program =
        VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default()).expect("Proof");
    input[0] = kernel_support::integer(0);
    let outcome = program
        .execute(KernelExecutionBudget::default())
        .expect("Execution");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(private_value))
    );
    assert_eq!((outcome.steps, outcome.peak_stack_values), (2, 1));
    assert_eq!(program.usage().verification_steps, 5);
    assert_eq!(program.instructions().len(), 2);
    assert!(
        !format!("{program:?} {:?} {outcome:?}", program.instructions())
            .contains(&private_value.to_string())
    );
    let undefined = VerifiedKernelProgram::verify(
        &[
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Undefined return proof");
    assert_eq!(
        undefined
            .execute(KernelExecutionBudget::default())
            .expect("Undefined return")
            .value,
        KernelValue::Undefined
    );
}

#[test]
fn branch_paths_consume_numeric_truth_with_explicit_fuel_and_one_result() {
    for condition in [
        KernelBranchCondition::IfFalse,
        KernelBranchCondition::IfTrue,
    ] {
        for number in [
            LegacyNumber::Integer(0),
            LegacyNumber::Integer(-1),
            LegacyNumber::Float(f32::NAN),
        ] {
            let input = [
                KernelInstruction::Constant(KernelValue::Number(number)),
                KernelInstruction::Branch {
                    condition,
                    target: 4,
                },
                kernel_support::integer(7),
                KernelInstruction::Jump(5),
                kernel_support::integer(42),
                KernelInstruction::Return,
            ];
            let program =
                VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
                    .expect("Branch proof");
            let taken = match condition {
                KernelBranchCondition::IfFalse => !number.as_bool(),
                KernelBranchCondition::IfTrue => number.as_bool(),
            };
            let steps = if taken { 4 } else { 5 };
            let exact = KernelExecutionBudget {
                max_steps: steps,
                max_stack_values: 1,
                max_stack_bytes: size_of::<KernelValue>() as u64,
            };
            let outcome = program.execute(exact).expect("Exact fuel/stack");
            assert_eq!(
                outcome.value,
                KernelValue::Number(LegacyNumber::Integer(if taken { 42 } else { 7 }))
            );
            assert_eq!((outcome.steps, outcome.peak_stack_values), (steps, 1));
            assert_eq!(program.usage().verification_steps, 18);
            let error = program
                .execute(KernelExecutionBudget {
                    max_steps: steps - 1,
                    ..exact
                })
                .expect_err("One less fuel");
            assert_eq!(error.work_steps, steps - 1);
            assert_eq!(error.peak_stack_values, 1);
            assert_eq!(
                error.kind,
                KernelErrorKind::BudgetExceeded {
                    resource: KernelResource::ExecutionSteps,
                    requested: steps,
                    maximum: steps - 1
                }
            );
        }
    }
}

#[test]
fn stack_and_unary_instructions_execute_without_leaking_implicit_values() -> Result<(), String> {
    let program = VerifiedKernelProgram::verify(
        &[
            kernel_support::integer(7),
            KernelInstruction::Duplicate,
            KernelInstruction::Discard,
            KernelInstruction::BitwiseNot,
            KernelInstruction::LogicalNot,
            KernelInstruction::CastBool,
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Unary/stack proof");
    let outcome = program
        .execute(KernelExecutionBudget::default())
        .expect("Unary/stack execution");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(0)));
    assert_eq!((outcome.steps, outcome.peak_stack_values), (7, 2));
    let bits = 0x7f80_0001;
    let program = VerifiedKernelProgram::verify(
        &[
            KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Float(f32::from_bits(
                bits,
            )))),
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Raw float proof");
    let KernelValue::Number(LegacyNumber::Float(value)) = program
        .execute(KernelExecutionBudget::default())
        .expect("Raw float return")
        .value
    else {
        return Err("Expected raw binary32 return".to_owned());
    };
    assert_eq!(value.to_bits(), bits);
    Ok(())
}

#[test]
fn runtime_numeric_and_subset_type_errors_retain_instruction_and_charged_work() {
    let program = VerifiedKernelProgram::verify(
        &[
            kernel_support::integer(1_357_913_579),
            kernel_support::integer(0),
            KernelInstruction::Divide,
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Runtime-error proof");
    let error = program
        .execute(KernelExecutionBudget::default())
        .expect_err("Numeric error");
    assert_eq!(
        error.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    assert_eq!(
        (error.instruction, error.work_steps, error.peak_stack_values),
        (Some(2), 3, 2)
    );
    assert!(error.source().is_some());
    assert!(!format!("{error:?} {error}").contains("1357913579"));
    let undefined = VerifiedKernelProgram::verify(
        &[
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::CastBool,
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Subset type proof");
    let error = undefined
        .execute(KernelExecutionBudget::default())
        .expect_err("Numeric-only operation");
    assert_eq!(error.kind, KernelErrorKind::ExpectedNumber);
    assert_eq!(
        (error.instruction, error.work_steps, error.peak_stack_values),
        (Some(1), 2, 1)
    );
}
