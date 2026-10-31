// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{MAX_MESSAGE_BYTES, Route};

use crate::{ChannelError, Transmission};

/// Ordinary-channel sender with one outstanding borrowed transmission at a time.
#[derive(Debug)]
pub struct SendChannel {
    pub(crate) route: Route,
    next_sequence: u32,
}

impl SendChannel {
    /// Creates a sender using sequence one, following upstream channel setup.
    #[must_use]
    pub fn new(route: Route) -> Self {
        Self {
            route,
            next_sequence: 1,
        }
    }

    /// Returns the sequence that the next logical message will reserve.
    #[must_use]
    pub fn next_sequence(&self) -> u32 {
        self.next_sequence
    }

    /// Reserves a sequence and borrows one complete message for packetization.
    ///
    /// Reservation happens once at begin, so cancelling a partial transmission
    /// never reuses its sequence with different contents. The returned borrow
    /// prevents beginning another transmission until it is dropped.
    ///
    /// # Errors
    /// Rejects oversized messages and sequences near reserved transport markers.
    /// On error no sequence is reserved. Reconnect instead of wrapping sequences.
    pub fn begin<'channel, 'message>(
        &'channel mut self,
        body: &'message [u8],
    ) -> Result<Transmission<'channel, 'message>, ChannelError> {
        if body.len() > MAX_MESSAGE_BYTES {
            return Err(ChannelError::MessageTooLarge);
        }
        if self.next_sequence >= 0x7fff_fff0 {
            return Err(ChannelError::SequenceExhausted);
        }
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        Ok(Transmission {
            owner: self,
            body,
            sequence,
            offset: 0,
            finished: false,
        })
    }
}
