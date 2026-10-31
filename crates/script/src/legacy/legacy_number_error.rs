// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Numeric type failure, excluding source values from diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyNumberError {
    /// The operator requires an integer and does not coerce a float.
    ExpectedInteger,
    /// Division rejects integer zero and either signed float zero.
    DivisionByZero,
}

impl fmt::Display for LegacyNumberError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ExpectedInteger => "legacy script operator requires an integer",
            Self::DivisionByZero => "legacy script division by zero",
        })
    }
}

impl Error for LegacyNumberError {}
