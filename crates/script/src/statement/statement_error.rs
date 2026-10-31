// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Local statement-view failure without source bytes, labels, names, or paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyStatementError {
    /// The supplied node is not a statement or supported statement wrapper.
    NotStatement,
    /// A pinned child count/role does not match the supported production profile.
    ProfileMismatch,
}

impl fmt::Display for LegacyStatementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotStatement => "legacy syntax node is not a statement",
            Self::ProfileMismatch => "legacy statement grammar profile mismatch",
        })
    }
}

impl Error for LegacyStatementError {}
