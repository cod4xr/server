// SPDX-License-Identifier: AGPL-3.0-or-later
use std::str::SplitWhitespace;
pub mod evaluator;
use evaluator::{EvaluationFailure, HelperEvaluator, VectorEvaluator};

use cod4x_script::legacy::{
    LegacyComparison, LegacyNumber, LegacyVector, LegacyVectorError, LegacyVectorOperand,
    LegacyVectorOperation, LegacyVectorPair,
};

pub fn validate(line: &str) -> Result<&str, String> {
    validate_with::<HelperEvaluator>(line)
}

pub fn validate_index<E: VectorEvaluator>(line: &str) -> Result<bool, String> {
    let mut fields = line.split_whitespace();
    if word(&mut fields)? != "I" {
        return Err("Expected index fixture row".into());
    }
    let object = LegacyVector::from_components([
        f32::from_bits(hex(&mut fields)?),
        f32::from_bits(hex(&mut fields)?),
        f32::from_bits(hex(&mut fields)?),
    ]);
    let index = operand(&mut fields)?;
    let expected_error = match index {
        LegacyVectorOperand::Number(LegacyNumber::Integer(_)) => LegacyVectorError::IndexOutOfRange,
        LegacyVectorOperand::Number(LegacyNumber::Float(_)) | LegacyVectorOperand::Vector(_) => {
            LegacyVectorError::ExpectedIntegerIndex
        }
    };
    let success = match (word(&mut fields)?, E::index(object, index)) {
        ("ok", Ok(LegacyNumber::Float(value))) => {
            assert_eq!(value.to_bits(), hex(&mut fields)?);
            true
        }
        ("E2", Err(EvaluationFailure::Domain(error))) if error == expected_error => false,
        (_, Err(EvaluationFailure::Unexpected(message))) => return Err(message),
        _ => return Err("Original vector index outcome differs".into()),
    };
    if fields.next().is_some() {
        return Err("Unexpected index fixture suffix".into());
    }
    Ok(success)
}

pub fn validate_with<E: VectorEvaluator>(line: &str) -> Result<&str, String> {
    let mut fields = line.split_whitespace();
    let kind = word(&mut fields)?;
    match kind {
        "V" => {
            let numbers = [
                number(&mut fields)?,
                number(&mut fields)?,
                number(&mut fields)?,
            ];
            exact_vector(
                E::construct(numbers).map_err(|error| error.to_string())?,
                &mut fields,
            )?;
        }
        "P" => {
            let pair = pair(&mut fields)?;
            let (left, right) = pair.operands();
            exact_vector(left, &mut fields)?;
            exact_vector(right, &mut fields)?;
        }
        "A" => arithmetic::<E>(&mut fields)?,
        "C" => comparison::<E>(&mut fields)?,
        _ => return Err("Unknown vector fixture row".into()),
    }
    if fields.next().is_some() {
        return Err("Unexpected vector fixture suffix".into());
    }
    Ok(kind)
}

fn arithmetic<E: VectorEvaluator>(fields: &mut SplitWhitespace<'_>) -> Result<(), String> {
    let operator = match word(fields)? {
        "0" => LegacyVectorOperation::Add,
        "1" => LegacyVectorOperation::Subtract,
        "2" => LegacyVectorOperation::Multiply,
        "3" => LegacyVectorOperation::Divide,
        _ => return Err("Unknown vector operation".into()),
    };
    let result = E::arithmetic(operator, operand(fields)?, operand(fields)?);
    match (word(fields)?, result) {
        ("ok", Ok(result)) => arithmetic_vector(result, fields),
        ("E1", Err(EvaluationFailure::Domain(LegacyVectorError::UnsupportedMixedAddition)))
        | ("E2", Err(EvaluationFailure::Domain(LegacyVectorError::DivisionByZero))) => Ok(()),
        _ => Err("Original vector arithmetic outcome differs".into()),
    }
}

fn comparison<E: VectorEvaluator>(fields: &mut SplitWhitespace<'_>) -> Result<(), String> {
    let operator = match word(fields)? {
        "0" => LegacyComparison::Equal,
        "1" => LegacyComparison::NotEqual,
        "2" => LegacyComparison::Less,
        "3" => LegacyComparison::Greater,
        "4" => LegacyComparison::LessOrEqual,
        "5" => LegacyComparison::GreaterOrEqual,
        _ => return Err("Unknown vector comparison".into()),
    };
    let result = E::compare(operator, operand(fields)?, operand(fields)?);
    match (word(fields)?, result) {
        ("ok", Ok(result)) => {
            let expected = match word(fields)? {
                "0" => false,
                "1" => true,
                _ => return Err("Invalid comparison boolean".into()),
            };
            assert_eq!(result, expected);
            Ok(())
        }
        ("E1", Err(EvaluationFailure::Domain(LegacyVectorError::UnsupportedOrdering))) => Ok(()),
        _ => Err("Original vector comparison outcome differs".into()),
    }
}

fn pair(fields: &mut SplitWhitespace<'_>) -> Result<LegacyVectorPair, String> {
    LegacyVectorPair::coerce(operand(fields)?, operand(fields)?).map_err(|error| error.to_string())
}

fn operand(fields: &mut SplitWhitespace<'_>) -> Result<LegacyVectorOperand, String> {
    let kind = word(fields)?;
    let bits = [hex(fields)?, hex(fields)?, hex(fields)?];
    match kind {
        "V" => Ok(LegacyVectorOperand::Vector(LegacyVector::from_components(
            bits.map(f32::from_bits),
        ))),
        "I" if bits[1..] == [0, 0] => Ok(LegacyVectorOperand::Number(LegacyNumber::Integer(
            i32::from_ne_bytes(bits[0].to_ne_bytes()),
        ))),
        "F" if bits[1..] == [0, 0] => Ok(LegacyVectorOperand::Number(LegacyNumber::Float(
            f32::from_bits(bits[0]),
        ))),
        _ => Err("Invalid fixture operand".into()),
    }
}

fn number(fields: &mut SplitWhitespace<'_>) -> Result<LegacyNumber, String> {
    match operand(fields)? {
        LegacyVectorOperand::Number(number) => Ok(number),
        LegacyVectorOperand::Vector(_) => Err("Construction requires a number".into()),
    }
}

fn exact_vector(value: LegacyVector, fields: &mut SplitWhitespace<'_>) -> Result<(), String> {
    for actual in value.components() {
        assert_eq!(actual.to_bits(), hex(fields)?);
    }
    Ok(())
}

fn arithmetic_vector(value: LegacyVector, fields: &mut SplitWhitespace<'_>) -> Result<(), String> {
    for actual in value.components() {
        let expected = word(fields)?;
        if expected == "nan" {
            assert!(actual.is_nan());
        } else {
            assert_eq!(
                actual.to_bits(),
                u32::from_str_radix(expected, 16).map_err(|error| error.to_string())?
            );
        }
    }
    Ok(())
}

fn hex(fields: &mut SplitWhitespace<'_>) -> Result<u32, String> {
    u32::from_str_radix(word(fields)?, 16).map_err(|error| error.to_string())
}

fn word<'source>(fields: &mut SplitWhitespace<'source>) -> Result<&'source str, String> {
    fields
        .next()
        .ok_or_else(|| "Missing vector fixture field".into())
}
