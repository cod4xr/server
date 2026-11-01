// SPDX-License-Identifier: AGPL-3.0-or-later
//! Existing source-grounded scalar fixtures pass through owned verification/execution.
#![allow(
    clippy::expect_used,
    reason = "Tests decode hash-pinned synthetic native scalar fixtures."
)]

use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{
    LegacyArithmetic, LegacyComparison, LegacyIntegerOperator, LegacyNumber, LegacyNumberError,
};

use crate::kernel_support;

fn operation(code: usize) -> KernelInstruction {
    let operators = [
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
        KernelInstruction::Arithmetic(LegacyArithmetic::Multiply),
        KernelInstruction::Integer(LegacyIntegerOperator::Or),
        KernelInstruction::Integer(LegacyIntegerOperator::And),
        KernelInstruction::Integer(LegacyIntegerOperator::ExclusiveOr),
        KernelInstruction::Integer(LegacyIntegerOperator::Remainder),
        KernelInstruction::Integer(LegacyIntegerOperator::ShiftLeft),
        KernelInstruction::Integer(LegacyIntegerOperator::ShiftRight),
    ];
    operators[code]
}

fn result_matches(value: KernelValue, kind: &str, bits: &str) -> Result<(), String> {
    match value {
        KernelValue::Number(LegacyNumber::Integer(value)) => {
            assert_eq!(kind, "I");
            assert_eq!(
                value.cast_unsigned(),
                u32::from_str_radix(bits, 16).expect("Integer bits")
            );
        }
        KernelValue::Number(LegacyNumber::Float(value)) => {
            assert_eq!(kind, "F");
            if bits == "nan" {
                assert!(value.is_nan());
            } else {
                assert_eq!(
                    value.to_bits(),
                    u32::from_str_radix(bits, 16).expect("Float bits")
                );
            }
        }
        KernelValue::Undefined
        | KernelValue::Vector(_)
        | KernelValue::Function(_)
        | KernelValue::String(_)
        | KernelValue::LocalizedString(_) => {
            return Err("Expected numeric execution result".to_owned());
        }
    }
    Ok(())
}

#[test]
fn source_arithmetic_and_machine_profile_cases_match_the_executing_stack_kernel()
-> Result<(), String> {
    let mut count = 0;
    for row in include_str!("fixtures/arithmetic_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let left = kernel_support::number(fields[2], fields[3])?;
        let right = kernel_support::number(fields[4], fields[5])?;
        let input = [
            KernelInstruction::Constant(KernelValue::Number(left)),
            KernelInstruction::Constant(KernelValue::Number(right)),
            operation(fields[1].parse().expect("Opcode")),
            KernelInstruction::Return,
        ];
        let program = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
            .expect("Fixture proof");
        let result = program.execute(KernelExecutionBudget::default());
        if fields[6] == "0" {
            result_matches(result.expect("Source success").value, fields[7], fields[8])?;
        } else {
            let error = result.expect_err("Source numeric error");
            let expected = match fields[6] {
                "1" => LegacyNumberError::ExpectedInteger,
                "2" => LegacyNumberError::DivisionByZero,
                _ => return Err("Unknown source error".to_owned()),
            };
            assert_eq!(error.kind, KernelErrorKind::Numeric(expected));
            assert_eq!(
                (error.instruction, error.work_steps, error.peak_stack_values),
                (Some(2), 3, 2)
            );
        }
        count += 1;
    }
    assert_eq!(count, 11_519);
    Ok(())
}

#[test]
fn source_comparisons_and_divisions_match_numeric_instruction_dispatch() -> Result<(), String> {
    let comparisons = [
        LegacyComparison::Equal,
        LegacyComparison::NotEqual,
        LegacyComparison::Less,
        LegacyComparison::Greater,
        LegacyComparison::LessOrEqual,
        LegacyComparison::GreaterOrEqual,
    ];
    let mut count = 0;
    for row in include_str!("fixtures/numeric_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        if fields[0] == "P" {
            continue;
        } // Coercion-only records have no standalone kernel opcode.
        let (start, operation) = match fields[0] {
            "C" => (
                2,
                KernelInstruction::Compare(
                    comparisons[fields[1].parse::<usize>().expect("Comparison code")],
                ),
            ),
            "D" => (1, KernelInstruction::Divide),
            _ => return Err("Unknown numeric fixture operation".to_owned()),
        };
        let left = kernel_support::number(fields[start], fields[start + 1])?;
        let right = kernel_support::number(fields[start + 2], fields[start + 3])?;
        let input = [
            KernelInstruction::Constant(KernelValue::Number(left)),
            KernelInstruction::Constant(KernelValue::Number(right)),
            operation,
            KernelInstruction::Return,
        ];
        let program = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
            .expect("Numeric proof");
        let result = program.execute(KernelExecutionBudget::default());
        if fields[0] == "C" {
            assert_eq!(
                result.expect("Comparison execution").value,
                KernelValue::Number(LegacyNumber::Integer(
                    fields[6].parse().expect("Boolean result")
                ))
            );
        } else if fields[5] == "1" {
            assert_eq!(
                result.expect_err("Division by zero").kind,
                KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
            );
        } else {
            result_matches(result.expect("Division execution").value, "F", fields[6])?;
        }
        count += 1;
    }
    assert_eq!(count, 10_864);
    Ok(())
}
