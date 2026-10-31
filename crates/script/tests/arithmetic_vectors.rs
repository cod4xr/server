// SPDX-License-Identifier: AGPL-3.0-or-later
//! Defined original helper cases and separately identified archival instruction cases.
#![allow(
    clippy::expect_used,
    reason = "Tests own generated synthetic scalar/profile fixtures."
)]

use cod4x_script::legacy::{
    LegacyArithmetic, LegacyIntegerOperator, LegacyNumber, LegacyNumberError, LegacyNumericPair,
};

fn number(kind: &str, bits: &str) -> Result<LegacyNumber, String> {
    let bits = u32::from_str_radix(bits, 16).map_err(|_| "Invalid scalar bits")?;
    match kind {
        "I" => Ok(LegacyNumber::Integer(bits.cast_signed())),
        "F" => Ok(LegacyNumber::Float(f32::from_bits(bits))),
        _ => Err("Unknown scalar kind".to_owned()),
    }
}

fn evaluate(
    operation: usize,
    left: LegacyNumber,
    right: LegacyNumber,
) -> Result<LegacyNumber, LegacyNumberError> {
    let arithmetic = [
        LegacyArithmetic::Add,
        LegacyArithmetic::Subtract,
        LegacyArithmetic::Multiply,
    ];
    let integer = [
        LegacyIntegerOperator::Or,
        LegacyIntegerOperator::And,
        LegacyIntegerOperator::ExclusiveOr,
        LegacyIntegerOperator::Remainder,
        LegacyIntegerOperator::ShiftLeft,
        LegacyIntegerOperator::ShiftRight,
    ];
    if operation < 3 {
        Ok(arithmetic[operation].evaluate(LegacyNumericPair::coerce(left, right)))
    } else {
        integer[operation - 3].evaluate(left, right)
    }
}

#[test]
fn arithmetic_matches_defined_helpers_and_the_explicit_integer_instruction_profile()
-> Result<(), String> {
    let mut original = 0;
    let mut instruction = 0;
    for row in include_str!("fixtures/arithmetic_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let operation = fields[1].parse().expect("Operation");
        let left = number(fields[2], fields[3])?;
        let right = number(fields[4], fields[5])?;
        let result = evaluate(operation, left, right);
        match fields[6] {
            "0" => match result.expect("Source success") {
                LegacyNumber::Integer(value) => {
                    assert_eq!(fields[7], "I");
                    assert_eq!(
                        value.cast_unsigned(),
                        u32::from_str_radix(fields[8], 16).expect("Bits")
                    );
                }
                LegacyNumber::Float(value) => {
                    assert_eq!(fields[7], "F");
                    if fields[8] == "nan" {
                        assert!(value.is_nan());
                    } else {
                        assert_eq!(
                            value.to_bits(),
                            u32::from_str_radix(fields[8], 16).expect("Bits")
                        );
                    }
                }
            },
            "1" => assert_eq!(
                result.expect_err("Type error"),
                LegacyNumberError::ExpectedInteger
            ),
            "2" => assert_eq!(
                result.expect_err("Zero error"),
                LegacyNumberError::DivisionByZero
            ),
            _ => return Err("Unknown source error category".to_owned()),
        }
        match fields[0] {
            "C" => original += 1,
            "M" => {
                assert!(matches!(operation, 0..=2 | 7 | 8));
                assert_eq!((fields[2], fields[4], fields[6]), ("I", "I", "0"));
                instruction += 1;
            }
            _ => return Err("Unknown evidence origin".to_owned()),
        }
    }
    assert_eq!((original, instruction), (10_701, 818));
    Ok(())
}
