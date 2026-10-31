// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::legacy::LegacyNumberError;

/// Legacy GSC numeric scalars; not the future runtime's full value/handle model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyNumber {
    /// Signed 32-bit compatibility integer.
    Integer(i32),
    /// Binary32 compatibility float, preserving its input bits.
    Float(f32),
}

impl LegacyNumber {
    /// Returns numeric truth using the original nonzero comparison.
    ///
    /// Both signed zeros are false; NaNs/infinities are true. This does not permit
    /// arbitrary nonnumeric values or implement VM stack/refcount side effects.
    #[must_use]
    pub fn as_bool(self) -> bool {
        match self {
            Self::Integer(value) => value != 0,
            Self::Float(value) => value != 0.0,
        }
    }

    /// Applies legacy logical negation, producing an integer zero or one.
    #[must_use]
    pub fn logical_not(self) -> Self {
        Self::Integer(i32::from(!self.as_bool()))
    }

    /// Complements integer bits without accepting a float coercion.
    ///
    /// # Errors
    /// Returns `ExpectedInteger` for floats; no caller state is mutated.
    pub fn bitwise_not(self) -> Result<Self, LegacyNumberError> {
        match self {
            Self::Integer(value) => Ok(Self::Integer(!value)),
            Self::Float(_) => Err(LegacyNumberError::ExpectedInteger),
        }
    }
}
