// SPDX-License-Identifier: AGPL-3.0-or-later
//! Index operand order, type errors, resource limits and caller copies remain explicit.
#![allow(
    clippy::expect_used,
    reason = "Tests use finite typed primitive programs and exact failures."
)]

use crate::kernel_support;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelInstruction,
    KernelResource, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector, LegacyVectorError};

#[test]
fn index_stack_effect_and_exact_stack_fuel_limits_are_verified() -> Result<(), String> {
    for count in 0..2 {
        let mut code = vec![kernel_support::integer(0); count];
        code.extend([KernelInstruction::Index, KernelInstruction::Return]);
        assert_eq!(
            VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
                .expect_err("Two operands required")
                .kind,
            KernelErrorKind::StackUnderflow {
                required: 2,
                available: count as u64
            }
        );
    }
    let object = KernelValue::Vector(LegacyVector::from_components([1.0, -0.0, 3.0]));
    let code = [
        kernel_support::integer(1),
        KernelInstruction::Constant(object),
        KernelInstruction::Index,
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect("Index proof");
    let exact = KernelExecutionBudget {
        max_steps: 4,
        max_stack_values: 2,
        max_stack_bytes: 2 * size_of::<KernelValue>() as u64,
        max_local_values: 0,
        max_local_bytes: 0,
    };
    let outcome = program.execute(exact).expect("Exact index resources");
    let KernelValue::Number(LegacyNumber::Float(result)) = outcome.value else {
        return Err("Expected indexed float result".into());
    };
    assert_eq!(result.to_bits(), 0x8000_0000);
    for (resource, budget, work) in [
        (
            KernelResource::StackValues,
            KernelExecutionBudget {
                max_stack_values: 1,
                ..exact
            },
            0,
        ),
        (
            KernelResource::ExecutionStackBytes,
            KernelExecutionBudget {
                max_stack_bytes: exact.max_stack_bytes - 1,
                ..exact
            },
            0,
        ),
        (
            KernelResource::ExecutionSteps,
            KernelExecutionBudget {
                max_steps: 3,
                ..exact
            },
            3,
        ),
    ] {
        let error = program
            .execute(budget)
            .expect_err("One less index resource");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: actual, .. } if actual==resource)
        );
        assert_eq!(error.work_steps, work);
    }
    Ok(())
}

#[test]
fn indexed_parameters_stay_private_and_missing_or_bad_types_fail_without_values()
-> Result<(), String> {
    let code = [
        KernelInstruction::LoadLocal(1),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Index,
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(2, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Argument index proof");
    let vector = LegacyVector::from_components([f32::from_bits(0x7f80_0001), -0.0, 3.0]);
    let args = [
        KernelValue::Vector(vector),
        KernelValue::Number(LegacyNumber::Integer(0)),
    ];
    let KernelValue::Number(LegacyNumber::Float(result)) = program
        .execute_with_arguments(&args, KernelExecutionBudget::default())
        .expect("Copied coordinate")
        .value
    else {
        return Err("Expected float coordinate".into());
    };
    assert_eq!(result.to_bits(), 0x7f80_0001);
    let KernelValue::Vector(original) = args[0] else {
        return Err("Caller changed type".into());
    };
    assert_eq!(
        original.components().map(f32::to_bits),
        vector.components().map(f32::to_bits)
    );
    for (args, expected) in [
        (
            vec![KernelValue::Vector(vector)],
            KernelErrorKind::Vector(LegacyVectorError::ExpectedIntegerIndex),
        ),
        (vec![], KernelErrorKind::ExpectedIndexable),
        (
            vec![
                KernelValue::Number(LegacyNumber::Integer(1)),
                KernelValue::Undefined,
            ],
            KernelErrorKind::ExpectedIndexable,
        ),
        (
            vec![
                KernelValue::Vector(vector),
                KernelValue::Number(LegacyNumber::Integer(i32::MIN)),
            ],
            KernelErrorKind::Vector(LegacyVectorError::IndexOutOfRange),
        ),
    ] {
        let error = program
            .execute_legacy_script_arguments(&args, KernelExecutionBudget::default())
            .expect_err("Fixed index type/range error");
        assert_eq!(error.kind, expected);
        assert!(!format!("{error:?} {error}").contains("2147483648"));
    }
    Ok(())
}
