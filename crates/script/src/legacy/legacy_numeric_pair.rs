// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::legacy::{LegacyComparison, LegacyNumber, LegacyNumberError};

/// Coerced legacy numeric operands, without VM tags, references, or native IDs.
///
/// The selected profile uses binary32 evaluation and nearest rounding with FTZ/
/// DAZ disabled. This kernel does not set the host FP environment or model sticky
/// exception flags, VM unwinding, string/vector coercion, or modern numeric types.
#[derive(Clone, Copy)]
pub enum LegacyNumericPair {
    /// Two original signed compatibility integers.
    Integers(i32, i32),
    /// Two binary32 values; a mixed input converts only its integer operand.
    Floats(f32, f32),
}

impl LegacyNumericPair {
    /// Applies the numeric subset of the original weaker-pair coercion.
    ///
    /// No allocation/state change occurs. Same-kind values retain their original
    /// bits; mixed operands convert the integer to binary32 under this FP profile.
    #[must_use]
    pub fn coerce(left: LegacyNumber, right: LegacyNumber) -> Self {
        match (left, right) {
            (LegacyNumber::Integer(left), LegacyNumber::Integer(right)) => {
                Self::Integers(left, right)
            }
            (LegacyNumber::Float(left), LegacyNumber::Float(right)) => Self::Floats(left, right),
            (LegacyNumber::Integer(left), LegacyNumber::Float(right)) => {
                Self::Floats(binary32(left), right)
            }
            (LegacyNumber::Float(left), LegacyNumber::Integer(right)) => {
                Self::Floats(left, binary32(right))
            }
        }
    }

    /// Compares coerced numbers and returns the legacy integer zero/one result.
    ///
    /// Float equality is approximate and <=/>= negate strict predicates, including
    /// unordered NaNs. This is not Rust `PartialEq` and does not consume VM operands.
    #[must_use]
    pub fn compare(self, comparison: LegacyComparison) -> LegacyNumber {
        let result = match self {
            Self::Integers(left, right) => comparison.integers(left, right),
            Self::Floats(left, right) => comparison.floats(left, right),
        };
        LegacyNumber::Integer(i32::from(result))
    }

    /// Divides under the source profile, producing a float even for two integers.
    ///
    /// Overflow/infinities/NaNs follow binary32 operations. Arithmetic NaN payloads
    /// are unspecified; this API does not claim a full machine/VM floating ABI.
    ///
    /// # Errors
    /// Rejects integer zero or either float signed zero as `DivisionByZero`. No caller
    /// value is mutated; the original error-side default/stack cleanup is not modeled.
    pub fn divide(self) -> Result<LegacyNumber, LegacyNumberError> {
        let (left, right) = match self {
            Self::Integers(left, right) => (binary32(left), binary32(right)),
            Self::Floats(left, right) => (left, right),
        };
        if right == 0.0 {
            return Err(LegacyNumberError::DivisionByZero);
        }
        Ok(LegacyNumber::Float(left / right))
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "The source explicitly converts int32 to binary32 in the selected nearest profile."
)]
fn binary32(value: i32) -> f32 {
    value as f32
}

impl fmt::Debug for LegacyNumericPair {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Integers(_, _) => "LegacyNumericPair::Integers(..)",
            Self::Floats(_, _) => "LegacyNumericPair::Floats(..)",
        })
    }
}
