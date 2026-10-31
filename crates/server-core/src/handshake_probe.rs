// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::net::SocketAddr;
use std::str;
use std::time::Duration;

use cod4x_protocol::connection::{ChallengeResponse, ConnectRequest};
use cod4x_protocol::{CommandLine, Direction, MAX_COMMAND_BYTES, Packet};

use crate::query_budget::QueryBudget;
use crate::{DropReason, EndpointCookie, ProbeOutcome, ProbeReply};

/// Diagnostic legacy handshake exchange that never admits players or game state.
///
/// Issues stateless endpoint challenges and verifies echoed connect cookies, then
/// returns an honest gameplay-unavailable error. It never sends connectResponse,
/// contacts master/auth backends, authenticates an account, or allocates a client.
pub struct HandshakeProbe {
    cookies: EndpointCookie,
    budget: QueryBudget,
}

impl HandshakeProbe {
    /// Takes one process-owned cookie provider and starts empty reply budgets.
    #[must_use]
    pub fn new(cookies: EndpointCookie) -> Self {
        Self {
            cookies,
            budget: QueryBudget::default(),
        }
    }

    /// Handles one datagram under the discovery service's explicit reply budgets.
    ///
    /// Limits are four initial/per-second replies per IP, 100 global, 4096 budget
    /// entries, 30-second idle expiry, and at most one cleanup scan/second. Budget
    /// admission precedes MAC work. A bad-cookie connect also consumes budget but
    /// produces no reply. Malformed/unsupported syntax allocates no address state.
    /// All failures preserve output; input and supplied monotonic time are explicit.
    #[must_use]
    pub fn handle(
        &mut self,
        endpoint: SocketAddr,
        input: &[u8],
        output: &mut [u8],
        now: Duration,
    ) -> ProbeOutcome {
        let body = match Packet::parse(input, Direction::ClientToServer) {
            Ok(Packet::Connectionless(body)) => body,
            Ok(_) => return ProbeOutcome::Dropped(DropReason::Unsupported),
            Err(_) => return ProbeOutcome::Dropped(DropReason::Malformed),
        };
        let mut storage = [0; MAX_COMMAND_BYTES];
        let Ok(line) = CommandLine::read_connectionless(body, &mut storage) else {
            return ProbeOutcome::Dropped(DropReason::Malformed);
        };
        let mut tokens = line.tokens();
        let command = tokens.next().unwrap_or(&[]);
        if command.eq_ignore_ascii_case(b"getchallenge") {
            let nonce = match tokens.next() {
                None => 0,
                Some(bytes) => {
                    let Some(nonce) = str::from_utf8(bytes)
                        .ok()
                        .and_then(|text| text.trim_ascii().parse::<i32>().ok())
                    else {
                        return ProbeOutcome::Dropped(DropReason::InvalidChallenge);
                    };
                    nonce
                }
            };
            if !self.budget.allow(endpoint.ip(), now) {
                return ProbeOutcome::Dropped(DropReason::RateLimited);
            }
            let response = ChallengeResponse {
                server_challenge: self.cookies.issue(endpoint),
                client_challenge: nonce,
            };
            return match response.encode(output) {
                Ok(length) => ProbeOutcome::Reply {
                    kind: ProbeReply::Challenge,
                    length,
                },
                Err(_) => ProbeOutcome::Dropped(DropReason::OutputCapacity),
            };
        }
        if !command.eq_ignore_ascii_case(b"connect") {
            return ProbeOutcome::Dropped(DropReason::Unsupported);
        }
        let Ok(request) = ConnectRequest::parse(line) else {
            return ProbeOutcome::Dropped(DropReason::Malformed);
        };
        if !self.budget.allow(endpoint.ip(), now) {
            return ProbeOutcome::Dropped(DropReason::RateLimited);
        }
        if !self.cookies.verify(endpoint, request.challenge()) {
            return ProbeOutcome::Dropped(DropReason::InvalidChallenge);
        }
        let response =
            Packet::Connectionless(b"error\nRust development server is not ready for gameplay.\n");
        match response.encode(output) {
            Ok(length) => ProbeOutcome::Reply {
                kind: ProbeReply::Unavailable,
                length,
            },
            Err(_) => ProbeOutcome::Dropped(DropReason::OutputCapacity),
        }
    }
}

impl fmt::Debug for HandshakeProbe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HandshakeProbe { .. }")
    }
}
