// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic server services driven by explicit inputs and monotonic time.
//! Socket I/O, process state, signals, and log installation belong to applications.

mod discovery;
mod endpoint_cookie;
mod handshake_probe;
mod probe_outcome;
mod query_budget;
mod query_types;
mod token_bucket;

pub use discovery::DiscoveryService;
pub use endpoint_cookie::{CookieKeyError, ENDPOINT_COOKIE_KEY_BYTES, EndpointCookie};
pub use handshake_probe::HandshakeProbe;
pub use probe_outcome::{ProbeOutcome, ProbeReply};
pub use query_types::{DropReason, QueryKind, QueryOutcome};
