// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{ChannelPacket, FRAGMENT_BYTES, Packet};

use crate::{ChannelError, SendChannel};

/// One logical message with caller-controlled datagram scheduling and retries.
///
/// No socket I/O or pacing occurs here. Dropping an unfinished transmission
/// cancels it; its sequence remains reserved to prevent message mixing.
#[derive(Debug)]
pub struct Transmission<'channel, 'message> {
    pub(crate) owner: &'channel mut SendChannel,
    pub(crate) body: &'message [u8],
    pub(crate) sequence: u32,
    pub(crate) offset: usize,
    pub(crate) finished: bool,
}

impl Transmission<'_, '_> {
    /// Encodes the next datagram, returning `None` once the message is complete.
    ///
    /// A failed encoding leaves output and position unchanged, permitting retry
    /// with adequate storage. Successful encoding advances packetization even if
    /// the caller's later socket send fails, as with ordinary unreliable UDP.
    ///
    /// # Errors
    /// Returns a typed packet error when output cannot fit the complete datagram.
    pub fn next_packet(&mut self, output: &mut [u8]) -> Result<Option<usize>, ChannelError> {
        if self.finished {
            return Ok(None);
        }
        let fragmented = self.body.len() >= FRAGMENT_BYTES;
        let end = (self.offset + FRAGMENT_BYTES).min(self.body.len());
        let fragment_start = if fragmented {
            Some(u32::try_from(self.offset).map_err(|_| ChannelError::MessageTooLarge)?)
        } else {
            None
        };
        let packet = ChannelPacket::try_new(
            self.sequence,
            self.owner.route,
            fragment_start,
            &self.body[self.offset..end],
        )?;
        let length = Packet::Channel(packet).encode(output)?;
        self.finished = !fragmented || end - self.offset < FRAGMENT_BYTES;
        self.offset = end;
        Ok(Some(length))
    }
}
