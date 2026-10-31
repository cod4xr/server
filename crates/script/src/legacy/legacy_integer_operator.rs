// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::legacy::{LegacyNumber, LegacyNumberError};

/// Integer-only legacy operators, distinct from numeric pair coercion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyIntegerOperator {
    /// Bitwise inclusive OR.
    Or,
    /// Bitwise AND.
    And,
    /// Bitwise exclusive OR.
    ExclusiveOr,
    /// Signed remainder truncated toward zero; not Euclidean modulo.
    Remainder,
    /// x86-profile left shift using the low five count bits and discarding overflow.
    ShiftLeft,
    /// x86-profile arithmetic right shift using the low five count bits.
    ShiftRight,
}

impl LegacyIntegerOperator {
    /// Evaluates without coercing floats, allocating, or changing caller operands.
    ///
    /// Shift counts, including negative values, use the recorded instruction
    /// profile rather than undefined C shift behavior. Result integers remain
    /// compatibility scalars, never engine object/asset/function identifiers.
    ///
    /// # Errors
    /// Rejects either noninteger operand, zero remainder denominators, and MIN%-1.
    /// The last case becomes a controlled error instead of the native IDIV fault.
    pub fn evaluate(
        self,
        left: LegacyNumber,
        right: LegacyNumber,
    ) -> Result<LegacyNumber, LegacyNumberError> {
        let (LegacyNumber::Integer(left), LegacyNumber::Integer(right)) = (left, right) else {
            return Err(LegacyNumberError::ExpectedInteger);
        };
        let result = match self {
            Self::Or => left | right,
            Self::And => left & right,
            Self::ExclusiveOr => left ^ right,
            Self::ShiftLeft => left.wrapping_shl(right.cast_unsigned()),
            Self::ShiftRight => left.wrapping_shr(right.cast_unsigned()),
            Self::Remainder => {
                if right == 0 {
                    return Err(LegacyNumberError::DivisionByZero);
                }
                left.checked_rem(right)
                    .ok_or(LegacyNumberError::RemainderOverflow)?
            }
        };
        Ok(LegacyNumber::Integer(result))
    }
}
