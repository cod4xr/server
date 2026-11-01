// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{ChallengeError, challenge_decimal};
use crate::{CommandLine, Direction, Packet, PacketError};

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
    /// Parses a complete OOB datagram under this fixed challenge profile.
    /// One terminal newline/NUL is permitted, while bytes after it reject.
    /// This does not validate the peer or compare the caller's pending nonce.
    ///
    /// # Errors
    /// Rejects invalid envelopes, trailing data and checked command/profile errors.
    pub fn parse_packet(packet: &[u8]) -> Result<Self, ChallengeError> {
        let Packet::Connectionless(body) = Packet::parse(packet, Direction::ServerToClient)
            .map_err(|_| ChallengeError::InvalidEnvelope)?
        else {
            return Err(ChallengeError::InvalidEnvelope);
        };
        let line = CommandLine::parse(body).map_err(|_| ChallengeError::InvalidEnvelope)?;
        if line.consumed_bytes() != body.len() {
            return Err(ChallengeError::TrailingData);
        }
        Self::parse(line)
    }

    /// Parses the pinned `0 xproto 18` response profile after OOB line decoding.
    /// Both signed values must be complete decimal i32 tokens. Client runtimes
    /// must separately check the endpoint, echoed nonce, state and deadline.
    /// The suffix is a compatibility marker, not a negotiated gameplay version.
    ///
    /// # Errors
    /// Rejects missing/extra fields, malformed integers and unsupported suffixes.
    pub fn parse(line: CommandLine<'_>) -> Result<Self, ChallengeError> {
        let mut tokens = line.tokens();
        if !tokens
            .next()
            .is_some_and(|command| command.eq_ignore_ascii_case(b"challengeResponse"))
        {
            return Err(ChallengeError::UnexpectedCommand);
        }
        let server = tokens.next().ok_or(ChallengeError::FieldCount)?;
        let client = tokens.next().ok_or(ChallengeError::FieldCount)?;
        let reserved = tokens.next().ok_or(ChallengeError::FieldCount)?;
        let marker = tokens.next().ok_or(ChallengeError::FieldCount)?;
        let hint = tokens.next().ok_or(ChallengeError::FieldCount)?;
        if tokens.next().is_some() {
            return Err(ChallengeError::FieldCount);
        }
        if reserved != b"0" || !marker.eq_ignore_ascii_case(b"xproto") || hint != b"18" {
            return Err(ChallengeError::UnsupportedProfile);
        }
        Ok(Self {
            server_challenge: challenge_decimal::parse(server)?,
            client_challenge: challenge_decimal::parse(client)?,
        })
    }

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
            challenge_decimal::encode(self.server_challenge, &mut server),
            b" ",
            challenge_decimal::encode(self.client_challenge, &mut client),
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

impl fmt::Debug for ChallengeResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ChallengeResponse { .. }")
    }
}
