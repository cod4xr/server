// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::legacy::{LegacyArithmetic, LegacyNumber, LegacyNumberError, LegacyNumericPair};

/// Legacy integer-only update under the guarded x86 32-bit word profile.
///
/// Source ++/-- statements reject floats rather than coercing them. Richer modern
/// numeric rules and native variable-reference/error-unwinding behavior are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyIntegerUpdate {
    /// Adds integer one, wrapping at the int32 boundary.
    Increment,
    /// Subtracts integer one, wrapping at the int32 boundary.
    Decrement,
}

impl LegacyIntegerUpdate {
    /// Evaluates an integer without allocation, mutation, or float conversion.
    ///
    /// # Errors
    /// Rejects every float, including integral values, signed zero, infinities and NaNs.
    /// Errors retain no operand contents; integer overflow wraps in debug/release.
    pub fn evaluate(self, value: LegacyNumber) -> Result<LegacyNumber, LegacyNumberError> {
        let LegacyNumber::Integer(value) = value else {
            return Err(LegacyNumberError::ExpectedInteger);
        };
        let arithmetic = match self {
            Self::Increment => LegacyArithmetic::Add,
            Self::Decrement => LegacyArithmetic::Subtract,
        };
        Ok(arithmetic.evaluate(LegacyNumericPair::Integers(value, 1)))
    }
}
