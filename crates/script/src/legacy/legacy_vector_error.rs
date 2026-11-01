// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Fixed vector-domain failure, without coordinate data, source or native cleanup state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyVectorError {
    /// This API requires at least one vector; scalar-only pairs use the scalar API.
    ExpectedVectorPair,
    /// Legacy addition's string-aware coercer rejects a mixed numeric/vector pair.
    UnsupportedMixedAddition,
    /// Vector comparisons support only equality/inequality in this profile.
    UnsupportedOrdering,
    /// At least one denominator coordinate is either signed zero.
    DivisionByZero,
    /// A vector index must retain the integer type; floats are not coerced.
    ExpectedIntegerIndex,
    /// An integer index falls outside the three-coordinate domain; its value is redacted.
    IndexOutOfRange,
}

impl fmt::Display for LegacyVectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "legacy vector error: {self:?}")
    }
}

impl Error for LegacyVectorError {}
