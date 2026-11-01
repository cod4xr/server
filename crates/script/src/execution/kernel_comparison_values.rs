// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelErrorKind, KernelValue};
use crate::legacy::{
    LegacyComparison, LegacyNumber, LegacyNumericPair, LegacyVectorOperand, LegacyVectorPair,
};

pub(super) fn compare(
    comparison: LegacyComparison,
    left: KernelValue,
    right: KernelValue,
) -> Result<KernelValue, KernelErrorKind> {
    let equal = match (left, right) {
        (KernelValue::Undefined, KernelValue::Undefined) => match comparison {
            LegacyComparison::Equal => true,
            LegacyComparison::NotEqual => false,
            _ => return Err(KernelErrorKind::UnmatchingTypes),
        },
        (KernelValue::Undefined, _) | (_, KernelValue::Undefined) => {
            return Err(KernelErrorKind::UnmatchingTypes);
        }
        (KernelValue::Number(left), KernelValue::Number(right)) => {
            return Ok(KernelValue::Number(
                LegacyNumericPair::coerce(left, right).compare(comparison),
            ));
        }
        (left, right) => LegacyVectorPair::coerce(operand(left)?, operand(right)?)
            .and_then(|pair| pair.compare(comparison))
            .map_err(KernelErrorKind::Vector)?,
    };
    Ok(KernelValue::Number(LegacyNumber::Integer(i32::from(equal))))
}

fn operand(value: KernelValue) -> Result<LegacyVectorOperand, KernelErrorKind> {
    match value {
        KernelValue::Number(value) => Ok(LegacyVectorOperand::Number(value)),
        KernelValue::Vector(value) => Ok(LegacyVectorOperand::Vector(value)),
        KernelValue::Undefined => Err(KernelErrorKind::UnmatchingTypes),
    }
}
