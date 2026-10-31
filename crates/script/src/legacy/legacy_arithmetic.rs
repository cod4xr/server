// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::legacy::{LegacyNumber, LegacyNumericPair};

/// Scalar arithmetic with explicit int32 wrapping and binary32 float evaluation.
///
/// Wrapping follows the recorded x86 instruction profile, including cases where
/// signed C arithmetic is undefined. Modern numeric types remain separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyArithmetic {
    /// Adds coerced operands, without string/vector conversion.
    Add,
    /// Subtracts the right operand from the left.
    Subtract,
    /// Multiplies coerced operands, without vector conversion.
    Multiply,
}

impl LegacyArithmetic {
    /// Evaluates an already-coerced scalar pair without allocation or mutation.
    ///
    /// Float evaluation uses the pair's explicit nearest/binary32 profile. NaN
    /// payloads and host floating-point status/control state are not guaranteed.
    /// Integer overflow wraps identically in debug/release builds.
    #[must_use]
    pub fn evaluate(self, pair: LegacyNumericPair) -> LegacyNumber {
        match pair {
            LegacyNumericPair::Integers(left, right) => LegacyNumber::Integer(match self {
                Self::Add => left.wrapping_add(right),
                Self::Subtract => left.wrapping_sub(right),
                Self::Multiply => left.wrapping_mul(right),
            }),
            LegacyNumericPair::Floats(left, right) => LegacyNumber::Float(match self {
                Self::Add => left + right,
                Self::Subtract => left - right,
                Self::Multiply => left * right,
            }),
        }
    }
}
