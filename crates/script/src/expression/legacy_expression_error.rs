// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Local expression-view failure without source bytes, identifiers, or host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyExpressionError {
    /// The supplied structural node is not an expression or primary production.
    NotExpression,
    /// A supported production differs from the pinned child-count/operator profile.
    ProfileMismatch,
}

impl fmt::Display for LegacyExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotExpression => "legacy syntax node is not an expression",
            Self::ProfileMismatch => "legacy expression grammar profile mismatch",
        })
    }
}

impl Error for LegacyExpressionError {}
