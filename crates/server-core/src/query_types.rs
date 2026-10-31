// SPDX-License-Identifier: AGPL-3.0-or-later
/// Supported discovery query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryKind {
    /// Lightweight server metadata.
    Info,
    /// Server metadata and player list, currently empty in query-only mode.
    Status,
}

/// Safe, low-cardinality reason for dropping a datagram; contains no peer data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DropReason {
    /// Envelope or command framing is invalid or oversized.
    Malformed,
    /// Packet transport or command is outside the implemented discovery service.
    Unsupported,
    /// Query echo would exceed limits or contain forbidden characters.
    InvalidChallenge,
    /// Per-address/global/state-capacity budget is exhausted or time regressed.
    RateLimited,
    /// Caller output cannot contain a full response.
    OutputCapacity,
}

/// Result of bounded discovery processing. Output is usable only on `Reply`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum QueryOutcome {
    /// A full response was written into the caller's output prefix.
    Reply {
        /// Kind of query answered.
        kind: QueryKind,
        /// Length of the encoded response.
        length: usize,
    },
    /// No response should be sent; caller output remains unchanged.
    Dropped(DropReason),
}
