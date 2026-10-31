// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Wait conversion failure without private script values or source text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyWaitError {
    /// Rate must be positive and representable by the source's signed integer.
    InvalidFrameRate,
    /// A wait duration is negative.
    Negative,
    /// Input is NaN or infinite, with no valid legacy timer interpretation.
    NonFinite,
    /// Product/rounded ticks exceed the valid legacy signed-integer domain.
    OutOfRange,
}

impl fmt::Display for LegacyWaitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidFrameRate => "invalid legacy script frame rate",
            Self::Negative => "negative script wait",
            Self::NonFinite => "non-finite script wait",
            Self::OutOfRange => "legacy script wait is out of range",
        })
    }
}

impl Error for LegacyWaitError {}
