// SPDX-License-Identifier: AGPL-3.0-or-later
use std::net::IpAddr;
use std::time::Duration;

use cod4x_protocol::{
    CommandLine, Direction, InfoLimit, InfoWriter, MAX_COMMAND_BYTES, MAX_PACKET_BYTES, Packet,
};

use crate::query_budget::QueryBudget;
use crate::{DropReason, QueryKind, QueryOutcome};

/// Bounded query-only discovery with explicit resource limits and no socket I/O.
///
/// Reports zero client slots and an explicit development identity. It neither
/// claims an active map nor accepts gameplay, authentication, or remote commands.
#[derive(Debug, Default)]
pub struct DiscoveryService {
    budget: QueryBudget,
}

impl DiscoveryService {
    /// Handles one input datagram at caller-supplied monotonic time.
    ///
    /// Query challenge echoes are capped at 128 bytes. Addresses share a global
    /// budget of 100 initial replies and 100/s refill, plus four initial replies
    /// and four/s per IP. At most 4096 IP entries survive, with 30-second idle
    /// expiry and at most one full cleanup scan per second. Mapped IPv4-in-IPv6
    /// addresses share the corresponding IPv4 budget. Unsupported/malformed
    /// packets do not allocate per-address state or generate error responses.
    ///
    /// Output is unchanged on every dropped result. Rate budget can be consumed
    /// even when output capacity is insufficient; callers should supply 1400 bytes.
    #[must_use]
    pub fn handle(
        &mut self,
        address: IpAddr,
        input: &[u8],
        output: &mut [u8],
        now: Duration,
    ) -> QueryOutcome {
        let body = match Packet::parse(input, Direction::ClientToServer) {
            Ok(Packet::Connectionless(body)) => body,
            Ok(_) => return QueryOutcome::Dropped(DropReason::Unsupported),
            Err(_) => return QueryOutcome::Dropped(DropReason::Malformed),
        };
        let mut storage = [0; MAX_COMMAND_BYTES];
        let Ok(command) = CommandLine::read_connectionless(body, &mut storage) else {
            return QueryOutcome::Dropped(DropReason::Malformed);
        };
        let mut tokens = command.tokens();
        let kind = match tokens.next() {
            Some(name) if name.eq_ignore_ascii_case(b"getinfo") => QueryKind::Info,
            Some(name) if name.eq_ignore_ascii_case(b"getstatus") => QueryKind::Status,
            Some(_) | None => return QueryOutcome::Dropped(DropReason::Unsupported),
        };
        let challenge = tokens.next().unwrap_or(&[]);
        if challenge.len() > 128 {
            return QueryOutcome::Dropped(DropReason::InvalidChallenge);
        }
        let mut info_storage = [0; 1024];
        let mut info = InfoWriter::new(&mut info_storage, InfoLimit::Normal);
        if info.set(b"challenge", challenge).is_err() {
            return QueryOutcome::Dropped(DropReason::InvalidChallenge);
        }
        for (key, value) in [
            (
                b"hostname".as_slice(),
                b"CoD4X Rust (query-only development)".as_slice(),
            ),
            (b"protocol", b"6"),
            (b"shortversion", b"rust-dev"),
            (b"clients", b"0"),
            (b"g_humanplayers", b"0"),
            (b"sv_maxclients", b"0"),
        ] {
            if info.set(key, value).is_err() {
                return QueryOutcome::Dropped(DropReason::OutputCapacity);
            }
        }
        if !self.budget.allow(address, now) {
            return QueryOutcome::Dropped(DropReason::RateLimited);
        }
        let mut body_storage = [0; MAX_PACKET_BYTES];
        let header = match kind {
            QueryKind::Info => b"infoResponse\n".as_slice(),
            QueryKind::Status => b"statusResponse\n".as_slice(),
        };
        let length = header.len() + info.as_bytes().len();
        body_storage[..header.len()].copy_from_slice(header);
        body_storage[header.len()..length].copy_from_slice(info.as_bytes());
        let length = if kind == QueryKind::Status {
            body_storage[length] = b'\n';
            length + 1
        } else {
            length
        };
        match Packet::Connectionless(&body_storage[..length]).encode(output) {
            Ok(length) => QueryOutcome::Reply { kind, length },
            Err(_) => QueryOutcome::Dropped(DropReason::OutputCapacity),
        }
    }
}
