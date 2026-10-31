// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integer-only updates preserve word boundaries and checked unary kernel execution.
#![allow(
    clippy::expect_used,
    reason = "Tests use finite typed update programs and explicit numeric boundaries."
)]

use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelResource, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyIntegerUpdate, LegacyNumber, LegacyNumberError};

#[test]
fn updates_wrap_integer_boundaries_and_reject_every_float_class() {
    for (value, increment, decrement) in [
        (i32::MIN, i32::MIN + 1, i32::MAX),
        (i32::MIN + 1, i32::MIN + 2, i32::MIN),
        (-2, -1, -3),
        (-1, 0, -2),
        (0, 1, -1),
        (1, 2, 0),
        (i32::MAX - 1, i32::MAX, i32::MAX - 2),
        (i32::MAX, i32::MIN, i32::MAX - 1),
    ] {
        assert_eq!(
            LegacyIntegerUpdate::Increment.evaluate(LegacyNumber::Integer(value)),
            Ok(LegacyNumber::Integer(increment))
        );
        assert_eq!(
            LegacyIntegerUpdate::Decrement.evaluate(LegacyNumber::Integer(value)),
            Ok(LegacyNumber::Integer(decrement))
        );
    }
    for bits in [
        0,
        0x8000_0000,
        0x3f80_0000,
        0xbf80_0000,
        0x7f80_0000,
        0xff80_0000,
        0x7fc1_2345,
        0x7f7f_ffff,
        1,
    ] {
        for update in [
            LegacyIntegerUpdate::Increment,
            LegacyIntegerUpdate::Decrement,
        ] {
            assert_eq!(
                update.evaluate(LegacyNumber::Float(f32::from_bits(bits))),
                Err(LegacyNumberError::ExpectedInteger)
            );
        }
    }
}

#[test]
fn unary_update_stack_proof_fuel_and_private_type_errors_remain_accounted() {
    for (update, value, expected) in [
        (LegacyIntegerUpdate::Increment, i32::MAX, i32::MIN),
        (LegacyIntegerUpdate::Decrement, i32::MIN, i32::MAX),
    ] {
        let input = [
            KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Integer(value))),
            KernelInstruction::IntegerUpdate(update),
            KernelInstruction::Return,
        ];
        let program = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
            .expect("Unary proof");
        assert_eq!(program.usage().max_stack_values, 1);
        let exact = KernelExecutionBudget {
            max_steps: 3,
            max_stack_values: 1,
            max_stack_bytes: size_of::<KernelValue>() as u64,
            max_local_values: 0,
            max_local_bytes: 0,
        };
        assert_eq!(
            program.execute(exact).expect("Exact unary execution").value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
        let error = program
            .execute(KernelExecutionBudget {
                max_steps: 2,
                ..exact
            })
            .expect_err("Return fuel");
        assert_eq!(error.work_steps, 2);
        assert!(matches!(
            error.kind,
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::ExecutionSteps,
                requested: 3,
                maximum: 2
            }
        ));
        for (value, expected) in [
            (KernelValue::Undefined, KernelErrorKind::ExpectedNumber),
            (
                KernelValue::Number(LegacyNumber::Float(1.0)),
                KernelErrorKind::Numeric(LegacyNumberError::ExpectedInteger),
            ),
        ] {
            let input = [
                KernelInstruction::Constant(value),
                KernelInstruction::IntegerUpdate(update),
                KernelInstruction::Return,
            ];
            let program =
                VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
                    .expect("Type errors are runtime errors");
            let error = program
                .execute(exact)
                .expect_err("Unsupported update operand");
            assert_eq!(error.kind, expected);
            assert_eq!(error.instruction, Some(1));
            assert_eq!((error.work_steps, error.peak_stack_values), (2, 1));
        }
    }
    let input = [
        KernelInstruction::IntegerUpdate(LegacyIntegerUpdate::Increment),
        KernelInstruction::Return,
    ];
    assert!(matches!(
        VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
            .expect_err("Unary underflow")
            .kind,
        KernelErrorKind::StackUnderflow {
            required: 1,
            available: 0
        }
    ));
}
