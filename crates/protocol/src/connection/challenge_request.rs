// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::challenge_decimal;
use crate::{Packet, PacketError};

/// A nonce-only getchallenge request for the pinned server's challenge handler.
/// No identity credential, authorization request or gameplay connect is encoded.
#[derive(Clone, Copy)]
pub struct ChallengeRequest {
    /// Signed random correlation value that must be echoed before advancing state.
    pub client_challenge: i32,
}

impl ChallengeRequest {
    /// Encodes the complete OOB request without a newline/NUL or credential fields.
    /// Output is unchanged on error; bytes beyond the encoded prefix are untouched.
    ///
    /// # Errors
    /// Rejects insufficient output through the shared packet codec.
    pub fn encode(self, output: &mut [u8]) -> Result<usize, PacketError> {
        let mut number = [0; 11];
        let nonce = challenge_decimal::encode(self.client_challenge, &mut number);
        let mut body = [0; 24];
        let prefix = b"getchallenge ";
        body[..prefix.len()].copy_from_slice(prefix);
        body[prefix.len()..prefix.len() + nonce.len()].copy_from_slice(nonce);
        Packet::Connectionless(&body[..prefix.len() + nonce.len()]).encode(output)
    }
}

impl fmt::Debug for ChallengeRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ChallengeRequest { .. }")
    }
}
