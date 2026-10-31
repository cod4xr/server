// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{ChannelPacket, FRAGMENT_BYTES, MAX_MESSAGE_BYTES, Route};

use crate::{ChannelError, ReceiveDrop, ReceiveOutcome, ReceivedMessage};

/// Bounded, single-owner ordinary-channel receive state.
///
/// Sequence advancement occurs only on complete messages. One partial message
/// is retained in caller storage; older in-flight fragments cannot overwrite it.
#[derive(Debug)]
pub struct ReceiveChannel<'buffer> {
    storage: &'buffer mut [u8],
    route: Route,
    incoming_sequence: u32,
    fragment_sequence: Option<u32>,
    fragment_length: usize,
}

impl<'buffer> ReceiveChannel<'buffer> {
    /// Creates a channel expecting the established peer direction and qport.
    ///
    /// The caller chooses reassembly capacity up to 128 KiB. No allocation occurs.
    ///
    /// # Errors
    /// Rejects storage above the logical message limit.
    pub fn try_new(storage: &'buffer mut [u8], route: Route) -> Result<Self, ChannelError> {
        if storage.len() > MAX_MESSAGE_BYTES {
            return Err(ChannelError::MessageTooLarge);
        }
        Ok(Self {
            storage,
            route,
            incoming_sequence: 0,
            fragment_sequence: None,
            fragment_length: 0,
        })
    }

    /// Returns the last delivered sequence, initially zero.
    #[must_use]
    pub fn incoming_sequence(&self) -> u32 {
        self.incoming_sequence
    }

    /// Returns bytes retained for the current partial message.
    #[must_use]
    pub fn partial_bytes(&self) -> usize {
        self.fragment_length
    }

    /// Discards partial reassembly, for example when a caller's timeout expires.
    /// It does not change the last delivered sequence or erase storage securely.
    pub fn discard_partial(&mut self) {
        self.fragment_sequence = None;
        self.fragment_length = 0;
    }

    /// Accepts one envelope already routed/authenticated by the caller.
    ///
    /// A full 1300-byte fragment remains pending until a shorter fragment arrives,
    /// including the required zero-byte final fragment for exact multiples.
    /// Duplicates, missing offsets, route mismatch, and capacity failures leave
    /// storage and state unchanged. Reordering is handled by ignoring an early
    /// fragment and accepting it if retransmitted after the missing prefix.
    #[must_use]
    pub fn accept<'message>(
        &'message mut self,
        packet: &'message ChannelPacket<'_>,
    ) -> ReceiveOutcome<'message> {
        if packet.route() != self.route {
            return ReceiveOutcome::Dropped(ReceiveDrop::WrongRoute);
        }
        let sequence = packet.sequence();
        if sequence <= self.incoming_sequence {
            return ReceiveOutcome::Dropped(ReceiveDrop::StaleSequence);
        }
        let length = if let Some(start) = packet.fragment_start() {
            if self
                .fragment_sequence
                .is_some_and(|active| sequence < active)
            {
                return ReceiveOutcome::Dropped(ReceiveDrop::StaleSequence);
            }
            let expected = if self.fragment_sequence == Some(sequence) {
                self.fragment_length
            } else {
                0
            };
            if usize::try_from(start).ok() != Some(expected) {
                return ReceiveOutcome::Dropped(ReceiveDrop::OutOfOrderFragment);
            }
            if packet.body().len() > self.storage.len() - expected {
                return ReceiveOutcome::Dropped(ReceiveDrop::Capacity);
            }
            let length = expected + packet.body().len();
            self.storage[expected..length].copy_from_slice(packet.body());
            self.fragment_sequence = Some(sequence);
            self.fragment_length = length;
            if packet.body().len() == FRAGMENT_BYTES {
                return ReceiveOutcome::Pending;
            }
            self.discard_partial();
            length
        } else {
            if packet.body().len() > self.storage.len() {
                return ReceiveOutcome::Dropped(ReceiveDrop::Capacity);
            }
            if self
                .fragment_sequence
                .is_some_and(|active| sequence >= active)
            {
                self.discard_partial();
            }
            let skipped_sequences = sequence - self.incoming_sequence - 1;
            self.incoming_sequence = sequence;
            return ReceiveOutcome::Message(ReceivedMessage {
                sequence,
                skipped_sequences,
                body: packet.body(),
            });
        };
        let skipped_sequences = sequence - self.incoming_sequence - 1;
        self.incoming_sequence = sequence;
        ReceiveOutcome::Message(ReceivedMessage {
            sequence,
            skipped_sequences,
            body: &self.storage[..length],
        })
    }
}
