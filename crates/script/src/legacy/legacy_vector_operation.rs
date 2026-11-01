// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::legacy::{LegacyVector, LegacyVectorError, LegacyVectorPair};

/// Legacy component-wise vector operations; multiplication is not a dot product.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyVectorOperation {
    /// Adds vector/vector coordinates; mixed original operands remain unsupported.
    Add,
    /// Subtracts coordinates after weaker-pair broadcast.
    Subtract,
    /// Multiplies coordinates after weaker-pair broadcast.
    Multiply,
    /// Divides coordinates after weaker-pair broadcast, rejecting any signed zero.
    Divide,
}

impl LegacyVectorOperation {
    /// Returns an owned result without modifying callers or allocating/refcounting.
    ///
    /// Float operations use the selected nearest/binary32 profile; arithmetic NaN
    /// payloads, host FP flags/environment and native error cleanup are not promised.
    ///
    /// # Errors
    /// Returns `UnsupportedMixedAddition` for mixed addition and `DivisionByZero`
    /// for any zero denominator. No native cleared intermediate result escapes.
    pub fn evaluate(self, pair: LegacyVectorPair) -> Result<LegacyVector, LegacyVectorError> {
        if self == Self::Add && pair.has_numeric_operand() {
            return Err(LegacyVectorError::UnsupportedMixedAddition);
        }
        let (left, right) = pair.operands();
        let left = left.components();
        let right = right.components();
        if self == Self::Divide && right.contains(&0.0) {
            return Err(LegacyVectorError::DivisionByZero);
        }
        let result = std::array::from_fn(|index| match self {
            Self::Add => left[index] + right[index],
            Self::Subtract => left[index] - right[index],
            Self::Multiply => left[index] * right[index],
            Self::Divide => left[index] / right[index],
        });
        Ok(LegacyVector::from_components(result))
    }
}
