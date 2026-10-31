// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::{Packet, PacketError};

/// The pinned server's legacy challenge response, without admission/authentication.
///
/// The fixed `0 xproto 18` suffix reproduces the source baseline. It does not
/// advertise custom capabilities or establish a negotiated gameplay protocol.
#[derive(Clone, Copy)]
pub struct ChallengeResponse {
    /// Opaque signed 32-bit return-path cookie issued by the server.
    pub server_challenge: i32,
    /// Echo of the client's signed nonce; old clients commonly send zero.
    pub client_challenge: i32,
}

impl ChallengeResponse {
    /// Encodes a complete connectionless datagram with no newline/NUL suffix.
    ///
    /// No allocation occurs. Output is unchanged on failure; unused output bytes
    /// remain untouched. This method does not issue or verify the cookie itself.
    ///
    /// # Errors
    /// Returns a packet capacity error if the complete response cannot fit.
    pub fn encode(self, output: &mut [u8]) -> Result<usize, PacketError> {
        let mut server = [0; 11];
        let mut client = [0; 11];
        let parts = [
            b"challengeResponse ".as_slice(),
            decimal(self.server_challenge, &mut server),
            b" ",
            decimal(self.client_challenge, &mut client),
            b" 0 xproto 18",
        ];
        // Two signed i32 decimals require at most 22 bytes; fixed text requires
        // another 30. No input can exceed this stack body's 64-byte capacity.
        let mut body = [0; 64];
        let mut length = 0;
        for part in parts {
            body[length..length + part.len()].copy_from_slice(part);
            length += part.len();
        }
        Packet::Connectionless(&body[..length]).encode(output)
    }
}

fn decimal(value: i32, storage: &mut [u8; 11]) -> &[u8] {
    let mut magnitude = value.unsigned_abs();
    let mut start = storage.len();
    loop {
        start -= 1;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "A decimal remainder is at most nine."
        )]
        let digit = (magnitude % 10) as u8;
        storage[start] = b'0' + digit;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        start -= 1;
        storage[start] = b'-';
    }
    &storage[start..]
}

impl fmt::Debug for ChallengeResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ChallengeResponse { .. }")
    }
}
