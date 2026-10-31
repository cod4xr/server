// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Owned plain-string operation failure without operand contents or native IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyStringError {
    /// The operation requires ordinary strings, not localized literals.
    ExpectedPlainString,
    /// Checked native result-length arithmetic overflowed.
    SizeOverflow,
    /// The configured logical result-byte budget was exceeded before allocation.
    BudgetExceeded {
        /// Complete result payload bytes, excluding an implicit terminator.
        requested: u64,
        /// Caller-supplied result payload budget.
        maximum: u64,
    },
    /// Fallible result allocation failed.
    AllocationFailed,
}

impl fmt::Display for LegacyStringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedPlainString => formatter.write_str("plain strings required"),
            Self::SizeOverflow => formatter.write_str("string result size overflow"),
            Self::BudgetExceeded { requested, maximum } => write!(
                formatter,
                "string result needs {requested} bytes; budget is {maximum}"
            ),
            Self::AllocationFailed => formatter.write_str("string result allocation failed"),
        }
    }
}

impl Error for LegacyStringError {}
