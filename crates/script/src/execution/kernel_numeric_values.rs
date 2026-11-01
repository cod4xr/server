// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelErrorKind, KernelInstruction, KernelValue};
use crate::legacy::{
    LegacyArithmetic, LegacyNumber, LegacyNumericPair, LegacyVector, LegacyVectorOperand,
    LegacyVectorOperation, LegacyVectorPair,
};

pub(super) fn binary(
    instruction: KernelInstruction,
    left: KernelValue,
    right: KernelValue,
) -> Result<KernelValue, KernelErrorKind> {
    if let (KernelValue::Number(left), KernelValue::Number(right)) = (left, right) {
        let pair = LegacyNumericPair::coerce(left, right);
        let value = match instruction {
            KernelInstruction::Arithmetic(operator) => operator.evaluate(pair),
            KernelInstruction::Divide => pair.divide().map_err(KernelErrorKind::Numeric)?,
            _ => return Err(KernelErrorKind::InvalidExecutionState),
        };
        return Ok(KernelValue::Number(value));
    }
    let pair = LegacyVectorPair::coerce(operand(left)?, operand(right)?)
        .map_err(KernelErrorKind::Vector)?;
    let operation = match instruction {
        KernelInstruction::Arithmetic(LegacyArithmetic::Add) => LegacyVectorOperation::Add,
        KernelInstruction::Arithmetic(LegacyArithmetic::Subtract) => {
            LegacyVectorOperation::Subtract
        }
        KernelInstruction::Arithmetic(LegacyArithmetic::Multiply) => {
            LegacyVectorOperation::Multiply
        }
        KernelInstruction::Divide => LegacyVectorOperation::Divide,
        _ => return Err(KernelErrorKind::InvalidExecutionState),
    };
    operation
        .evaluate(pair)
        .map(KernelValue::Vector)
        .map_err(KernelErrorKind::Vector)
}

pub(super) fn construct(values: [KernelValue; 3]) -> Result<KernelValue, KernelErrorKind> {
    let [x, y, z] = values;
    Ok(KernelValue::Vector(LegacyVector::from_numbers([
        number(x)?,
        number(y)?,
        number(z)?,
    ])))
}

fn number(value: KernelValue) -> Result<LegacyNumber, KernelErrorKind> {
    match value {
        KernelValue::Number(value) => Ok(value),
        KernelValue::Undefined | KernelValue::Vector(_) => Err(KernelErrorKind::ExpectedNumber),
    }
}

fn operand(value: KernelValue) -> Result<LegacyVectorOperand, KernelErrorKind> {
    match value {
        KernelValue::Number(value) => Ok(LegacyVectorOperand::Number(value)),
        KernelValue::Vector(value) => Ok(LegacyVectorOperand::Vector(value)),
        KernelValue::Undefined => Err(KernelErrorKind::ExpectedNumber),
    }
}
