// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::DropReason;

/// Response produced by the diagnostic handshake service; no players are admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeReply {
    /// An endpoint cookie and client nonce echo in the pinned legacy shape.
    Challenge,
    /// A valid-cookie connect was refused because gameplay is unavailable.
    Unavailable,
}

/// Bounded probe result. Output is unchanged unless a complete reply is produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProbeOutcome {
    /// Complete connectionless response in the caller's output prefix.
    Reply {
        /// Diagnostic response category; contains no supplied input values.
        kind: ProbeReply,
        /// Number of bytes to send.
        length: usize,
    },
    /// No reply should be sent; caller output remains unchanged.
    Dropped(DropReason),
}
