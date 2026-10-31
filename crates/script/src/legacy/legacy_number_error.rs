// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Scalar operator failure, excluding source values from diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyNumberError {
    /// The operator requires an integer and does not coerce a float.
    ExpectedInteger,
    /// Division/remainder rejects zero; float division rejects either signed zero.
    DivisionByZero,
    /// MIN%-1 would overflow the native signed quotient and fault its IDIV.
    RemainderOverflow,
}

impl fmt::Display for LegacyNumberError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ExpectedInteger => "legacy script operator requires an integer",
            Self::DivisionByZero => "legacy script division by zero",
            Self::RemainderOverflow => "legacy script remainder quotient overflow",
        })
    }
}

impl Error for LegacyNumberError {}
