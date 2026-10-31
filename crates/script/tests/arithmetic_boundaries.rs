// SPDX-License-Identifier: AGPL-3.0-or-later
//! Consistent overflow, masked shifts, controlled quotient faults, and scalar scope.
#![allow(
    clippy::expect_used,
    reason = "Tests supply synthetic integer and float operands."
)]

use cod4x_script::legacy::{
    LegacyArithmetic, LegacyIntegerOperator, LegacyNumber, LegacyNumberError, LegacyNumericPair,
};

#[test]
fn integer_arithmetic_wraps_without_build_mode_or_host_overflow_panics() {
    for (operation, left, right, expected) in [
        (LegacyArithmetic::Add, i32::MAX, 1, i32::MIN),
        (LegacyArithmetic::Add, i32::MIN, -1, i32::MAX),
        (LegacyArithmetic::Subtract, i32::MIN, 1, i32::MAX),
        (LegacyArithmetic::Subtract, i32::MAX, -1, i32::MIN),
        (LegacyArithmetic::Multiply, i32::MIN, -1, i32::MIN),
        (LegacyArithmetic::Multiply, 65_536, 65_536, 0),
    ] {
        assert_eq!(
            operation.evaluate(LegacyNumericPair::Integers(left, right)),
            LegacyNumber::Integer(expected)
        );
    }
}

#[test]
fn shifts_preserve_signed_bits_and_mask_negative_or_large_counts() {
    for (operation, left, right, expected) in [
        (LegacyIntegerOperator::ShiftLeft, 1, -1, i32::MIN),
        (LegacyIntegerOperator::ShiftLeft, -1, 31, i32::MIN),
        (LegacyIntegerOperator::ShiftLeft, 7, 32, 7),
        (LegacyIntegerOperator::ShiftLeft, 7, -32, 7),
        (LegacyIntegerOperator::ShiftLeft, 7, i32::MIN, 7),
        (LegacyIntegerOperator::ShiftRight, -3, 1, -2),
        (LegacyIntegerOperator::ShiftRight, i32::MIN, -1, -1),
        (LegacyIntegerOperator::ShiftRight, i32::MAX, i32::MAX, 0),
        (LegacyIntegerOperator::ShiftRight, -7, 32, -7),
        (LegacyIntegerOperator::ShiftRight, -7, -32, -7),
    ] {
        assert_eq!(
            operation
                .evaluate(LegacyNumber::Integer(left), LegacyNumber::Integer(right))
                .expect("Instruction profile"),
            LegacyNumber::Integer(expected)
        );
    }
}

#[test]
fn remainder_is_signed_and_native_quotient_faults_become_controlled_errors() {
    let operation = LegacyIntegerOperator::Remainder;
    for (left, right, expected) in [(-7, 3, -1), (7, -3, 1), (-7, -3, -1), (i32::MIN, 1, 0)] {
        assert_eq!(
            operation.evaluate(LegacyNumber::Integer(left), LegacyNumber::Integer(right)),
            Ok(LegacyNumber::Integer(expected))
        );
    }
    assert_eq!(
        operation.evaluate(LegacyNumber::Integer(i32::MIN), LegacyNumber::Integer(-1)),
        Err(LegacyNumberError::RemainderOverflow)
    );
    assert_eq!(
        operation.evaluate(LegacyNumber::Integer(7), LegacyNumber::Integer(0)),
        Err(LegacyNumberError::DivisionByZero)
    );
}

#[test]
fn integer_only_operators_reject_floats_before_zero_or_overflow_checks() {
    for operation in [
        LegacyIntegerOperator::Or,
        LegacyIntegerOperator::And,
        LegacyIntegerOperator::ExclusiveOr,
        LegacyIntegerOperator::Remainder,
        LegacyIntegerOperator::ShiftLeft,
        LegacyIntegerOperator::ShiftRight,
    ] {
        for (left, right) in [
            (LegacyNumber::Float(7.0), LegacyNumber::Integer(0)),
            (LegacyNumber::Integer(7), LegacyNumber::Float(0.0)),
            (LegacyNumber::Float(f32::NAN), LegacyNumber::Float(-0.0)),
        ] {
            assert_eq!(
                operation.evaluate(left, right),
                Err(LegacyNumberError::ExpectedInteger)
            );
        }
    }
}
