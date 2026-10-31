// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Local expression-view failure without source bytes, identifiers, or host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyExpressionError {
    /// The supplied structural node is not an expression or primary production.
    NotExpression,
    /// A valid expression uses an operator rather than a primary production.
    NotPrimary,
    /// A supported production differs from the pinned child-count/operator profile.
    ProfileMismatch,
}

impl fmt::Display for LegacyExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotExpression => "legacy syntax node is not an expression",
            Self::NotPrimary => "legacy expression is not a primary form",
            Self::ProfileMismatch => "legacy expression grammar profile mismatch",
        })
    }
}

impl Error for LegacyExpressionError {}
