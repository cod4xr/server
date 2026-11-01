// SPDX-License-Identifier: AGPL-3.0-or-later
use super::scope_fold_string;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, function_expression_lowering};
use crate::execution::KernelInstruction;
use crate::expression::LegacyInfixOperator;
use crate::legacy::{
    LegacyArithmetic, LegacyComparison, LegacyNumber, LegacyNumericPair, LegacyVector,
    LegacyVectorOperand, LegacyVectorOperation, LegacyVectorPair,
};
use crate::literal::LegacyString;
use std::ops::Range;

pub(super) enum FoldValue {
    Runtime,
    Number(LegacyNumber),
    Vector(LegacyVector),
    Undefined,
    String(LegacyString),
}

pub(super) fn release(
    builder: &mut FunctionBuilder<'_, '_>,
    value: FoldValue,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    if let FoldValue::String(value) = value {
        builder
            .ledger
            .release_control(value.as_bytes().len() as u64, span)?;
    }
    Ok(())
}

pub(super) fn vector(
    builder: &mut FunctionBuilder<'_, '_>,
    values: [FoldValue; 3],
    span: Range<u64>,
) -> Result<FoldValue, FunctionCompileError> {
    let result = match &values {
        [
            FoldValue::Number(x),
            FoldValue::Number(y),
            FoldValue::Number(z),
        ] => FoldValue::Vector(LegacyVector::from_numbers([*x, *y, *z])),
        _ => FoldValue::Runtime,
    };
    for value in values {
        release(builder, value, span.clone())?;
    }
    Ok(result)
}

pub(super) fn binary(
    builder: &mut FunctionBuilder<'_, '_>,
    operator: LegacyInfixOperator,
    left: FoldValue,
    right: FoldValue,
    span: Range<u64>,
) -> Result<FoldValue, FunctionCompileError> {
    let result = binary_value(builder, operator, &left, &right, span.clone())?;
    release(builder, left, span.clone())?;
    release(builder, right, span)?;
    Ok(result)
}

fn binary_value(
    builder: &mut FunctionBuilder<'_, '_>,
    operator: LegacyInfixOperator,
    left: &FoldValue,
    right: &FoldValue,
    span: Range<u64>,
) -> Result<FoldValue, FunctionCompileError> {
    if matches!(left, FoldValue::Runtime) || matches!(right, FoldValue::Runtime) {
        return Ok(FoldValue::Runtime);
    }
    let instruction = function_expression_lowering::infix(operator);
    if let (FoldValue::String(left), FoldValue::String(right)) = (left, right) {
        return scope_fold_string::binary(builder, instruction, left, right, span);
    }
    if let (FoldValue::Number(left), FoldValue::Number(right)) = (left, right) {
        let pair = LegacyNumericPair::coerce(*left, *right);
        let value = match instruction {
            KernelInstruction::Arithmetic(operator) => Some(operator.evaluate(pair)),
            KernelInstruction::Compare(comparison) => Some(pair.compare(comparison)),
            KernelInstruction::Divide => pair.divide().ok(),
            KernelInstruction::Integer(operator) => operator.evaluate(*left, *right).ok(),
            _ => None,
        };
        return Ok(value.map_or(FoldValue::Runtime, FoldValue::Number));
    }
    if let (FoldValue::Undefined, FoldValue::Undefined) = (left, right) {
        return Ok(match instruction {
            KernelInstruction::Compare(LegacyComparison::Equal) => {
                FoldValue::Number(LegacyNumber::Integer(1))
            }
            KernelInstruction::Compare(LegacyComparison::NotEqual) => {
                FoldValue::Number(LegacyNumber::Integer(0))
            }
            _ => FoldValue::Runtime,
        });
    }
    let Some((left, right)) = operand(left).zip(operand(right)) else {
        return Ok(FoldValue::Runtime);
    };
    let Ok(pair) = LegacyVectorPair::coerce(left, right) else {
        return Ok(FoldValue::Runtime);
    };
    let operation = match instruction {
        KernelInstruction::Arithmetic(LegacyArithmetic::Add) => LegacyVectorOperation::Add,
        KernelInstruction::Arithmetic(LegacyArithmetic::Subtract) => {
            LegacyVectorOperation::Subtract
        }
        KernelInstruction::Arithmetic(LegacyArithmetic::Multiply) => {
            LegacyVectorOperation::Multiply
        }
        KernelInstruction::Divide => LegacyVectorOperation::Divide,
        KernelInstruction::Compare(comparison) => {
            return Ok(pair
                .compare(comparison)
                .map_or(FoldValue::Runtime, |value| {
                    FoldValue::Number(LegacyNumber::Integer(i32::from(value)))
                }));
        }
        _ => return Ok(FoldValue::Runtime),
    };
    Ok(operation
        .evaluate(pair)
        .map_or(FoldValue::Runtime, FoldValue::Vector))
}

fn operand(value: &FoldValue) -> Option<LegacyVectorOperand> {
    match value {
        FoldValue::Number(number) => Some(LegacyVectorOperand::Number(*number)),
        FoldValue::Vector(vector) => Some(LegacyVectorOperand::Vector(*vector)),
        _ => None,
    }
}
