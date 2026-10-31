// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// A rejected channel transform, without key material or received contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ObfuscationError {
    /// Payload exceeds the current legacy compatibility message budget.
    MessageTooLarge,
    /// Acknowledged command is longer than the legacy command budget.
    CommandTooLong,
}

impl fmt::Display for ObfuscationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MessageTooLarge => "channel transform exceeds the message budget",
            Self::CommandTooLong => "channel transform command exceeds its size budget",
        })
    }
}

impl Error for ObfuscationError {}
