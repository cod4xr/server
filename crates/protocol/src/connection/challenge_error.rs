// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed challenge syntax/profile failures, without cookies or received contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChallengeError {
    /// The datagram is not a bounded connectionless command envelope.
    InvalidEnvelope,
    /// The command is not a challenge response.
    UnexpectedCommand,
    /// A required field is absent or additional command tokens are present.
    FieldCount,
    /// A cookie or nonce is outside the complete signed decimal i32 domain.
    InvalidInteger,
    /// The suffix is outside the explicitly supported `0 xproto 18` profile.
    UnsupportedProfile,
    /// Bytes remain after the command's newline/NUL terminator.
    TrailingData,
}

impl fmt::Display for ChallengeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "challenge rejected: {self:?}")
    }
}

impl Error for ChallengeError {}
