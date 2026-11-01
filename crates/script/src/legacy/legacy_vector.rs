// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::legacy::{LegacyNumber, LegacyVectorError};

/// Owned legacy binary32 triple, without native vector pools, references or VM IDs.
///
/// Construction preserves float input bits and converts integers under the selected
/// nearest/binary32 profile. This does not configure the host floating environment.
/// Derived equality is Rust data equality; language comparisons use explicit pairs.
#[derive(Clone, Copy, PartialEq)]
pub struct LegacyVector([f32; 3]);

impl LegacyVector {
    /// Copies already converted coordinates, preserving all IEEE input bits.
    #[must_use]
    pub const fn from_components(components: [f32; 3]) -> Self {
        Self(components)
    }

    /// Converts three numbers supplied in coordinate order, independently of native
    /// source-expression evaluation/push order or VM stack cleanup.
    #[must_use]
    pub fn from_numbers(numbers: [LegacyNumber; 3]) -> Self {
        Self(numbers.map(binary32))
    }

    /// Broadcasts one number into every coordinate for weaker-pair coercion.
    #[must_use]
    pub fn broadcast(number: LegacyNumber) -> Self {
        Self([binary32(number); 3])
    }

    /// Returns copied coordinate data; Debug output never prints these values.
    #[must_use]
    pub const fn components(self) -> [f32; 3] {
        self.0
    }

    /// Reads one coordinate as a float copy, preserving its exact input bits.
    ///
    /// # Errors
    /// Returns `ExpectedIntegerIndex` for floats and `IndexOutOfRange` outside 0..3.
    /// Errors never retain the attempted index or coordinates; callers stay unchanged.
    pub fn index(self, index: LegacyNumber) -> Result<LegacyNumber, LegacyVectorError> {
        let LegacyNumber::Integer(index) = index else {
            return Err(LegacyVectorError::ExpectedIntegerIndex);
        };
        let index = usize::try_from(index).map_err(|_| LegacyVectorError::IndexOutOfRange)?;
        self.0
            .get(index)
            .copied()
            .map(LegacyNumber::Float)
            .ok_or(LegacyVectorError::IndexOutOfRange)
    }
}

impl fmt::Debug for LegacyVector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("LegacyVector(..)")
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "The legacy numeric vector cast deliberately rounds int32 to binary32."
)]
fn binary32(number: LegacyNumber) -> f32 {
    match number {
        LegacyNumber::Integer(value) => value as f32,
        LegacyNumber::Float(value) => value,
    }
}
