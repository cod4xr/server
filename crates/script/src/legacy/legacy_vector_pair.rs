// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::legacy::{LegacyComparison, LegacyVector, LegacyVectorError, LegacyVectorOperand};

#[derive(Clone, Copy, Eq, PartialEq)]
enum PairKind {
    Vectors,
    LeftNumber,
    RightNumber,
}

/// Explicit weaker-coerced vector operands, retaining mixed-input identity for addition.
#[derive(Clone, Copy)]
pub struct LegacyVectorPair {
    left: LegacyVector,
    right: LegacyVector,
    kind: PairKind,
}

impl LegacyVectorPair {
    /// Copies vectors and broadcasts a numeric peer without allocation or mutation.
    ///
    /// # Errors
    /// Returns `ExpectedVectorPair` for scalar-only inputs, outside this API's domain.
    pub fn coerce(
        left: LegacyVectorOperand,
        right: LegacyVectorOperand,
    ) -> Result<Self, LegacyVectorError> {
        let (left, right, kind) = match (left, right) {
            (LegacyVectorOperand::Vector(left), LegacyVectorOperand::Vector(right)) => {
                (left, right, PairKind::Vectors)
            }
            (LegacyVectorOperand::Number(left), LegacyVectorOperand::Vector(right)) => {
                (LegacyVector::broadcast(left), right, PairKind::LeftNumber)
            }
            (LegacyVectorOperand::Vector(left), LegacyVectorOperand::Number(right)) => {
                (left, LegacyVector::broadcast(right), PairKind::RightNumber)
            }
            (LegacyVectorOperand::Number(_), LegacyVectorOperand::Number(_)) => {
                return Err(LegacyVectorError::ExpectedVectorPair);
            }
        };
        Ok(Self { left, right, kind })
    }

    /// Returns copied coerced coordinates in original operand order.
    #[must_use]
    pub const fn operands(self) -> (LegacyVector, LegacyVector) {
        (self.left, self.right)
    }

    /// Applies exact coordinate equality or its negation, including equal infinities,
    /// equal signed zeros and unequal NaNs. Scalar float tolerance does not apply.
    ///
    /// # Errors
    /// Returns `UnsupportedOrdering` for other comparison identities.
    #[expect(
        clippy::float_cmp,
        reason = "The original VectorCompare uses exact coordinate comparisons, unlike scalar equality."
    )]
    pub fn compare(self, operation: LegacyComparison) -> Result<bool, LegacyVectorError> {
        if !matches!(
            operation,
            LegacyComparison::Equal | LegacyComparison::NotEqual
        ) {
            return Err(LegacyVectorError::UnsupportedOrdering);
        }
        let left = self.left.components();
        let right = self.right.components();
        let equal = left[0] == right[0] && left[1] == right[1] && left[2] == right[2];
        Ok(if operation == LegacyComparison::Equal {
            equal
        } else {
            !equal
        })
    }

    pub(super) fn has_numeric_operand(self) -> bool {
        self.kind != PairKind::Vectors
    }
}

impl fmt::Debug for LegacyVectorPair {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("LegacyVectorPair(..)")
    }
}
