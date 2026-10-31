// SPDX-License-Identifier: AGPL-3.0-or-later
//! Socket-free channel sequencing and fragmentation with explicit buffer ownership.
//!
//! This crate handles ordinary channel envelopes, not authentication, reliable
//! commands, transport pacing, or the separate `CoD4X` reliable transport. Callers
//! must authenticate and route peer datagrams before supplying decoded envelopes.

mod channel_error;
mod receive_channel;
mod receive_outcome;
mod send_channel;
mod transmission;

pub use channel_error::ChannelError;
pub use receive_channel::ReceiveChannel;
pub use receive_outcome::{ReceiveDrop, ReceiveOutcome, ReceivedMessage};
pub use send_channel::SendChannel;
pub use transmission::Transmission;
