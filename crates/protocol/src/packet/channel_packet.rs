// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{FRAGMENT_BYTES, PacketError, Route};
use crate::MAX_MESSAGE_BYTES;

/// A validated normal-channel envelope, independent of socket or channel state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelPacket<'packet> {
    sequence: u32,
    route: Route,
    fragment_start: Option<u32>,
    body: &'packet [u8],
}

impl<'packet> ChannelPacket<'packet> {
    /// Validates a sequence, route, optional fragment offset, and borrowed body.
    ///
    /// This checks envelope bounds, not sequencing, authentication, or message
    /// semantics. A zero-length final fragment is valid.
    ///
    /// # Errors
    /// Rejects sequences above 31 bits, fragmented marker collisions, fragments
    /// above 1300 bytes, negative/unrepresentable offsets, or logical overflow.
    pub fn try_new(
        sequence: u32,
        route: Route,
        fragment_start: Option<u32>,
        body: &'packet [u8],
    ) -> Result<Self, PacketError> {
        if sequence > 0x7fff_ffff {
            return Err(PacketError::InvalidSequence);
        }
        if let Some(start) = fragment_start {
            if matches!(sequence, 0x7fff_fff0 | 0x7fff_fffe | 0x7fff_ffff) {
                return Err(PacketError::InvalidSequence);
            }
            let start = usize::try_from(start).map_err(|_| PacketError::InvalidFragment)?;
            if body.len() > FRAGMENT_BYTES
                || start > MAX_MESSAGE_BYTES
                || body.len() > MAX_MESSAGE_BYTES - start
            {
                return Err(PacketError::InvalidFragment);
            }
        }
        let header_length = match route {
            Route::ToServer { .. } => 6,
            Route::ToClient => 4,
        } + if fragment_start.is_some() { 6 } else { 0 };
        if body.len() > MAX_MESSAGE_BYTES - header_length {
            return Err(PacketError::InvalidFragment);
        }
        Ok(Self {
            sequence,
            route,
            fragment_start,
            body,
        })
    }

    /// Returns the sequence without the fragment flag.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Returns the direction and any client qport.
    #[must_use]
    pub fn route(&self) -> Route {
        self.route
    }

    /// Returns the offset for fragmented packets, or `None` for ordinary packets.
    #[must_use]
    pub fn fragment_start(&self) -> Option<u32> {
        self.fragment_start
    }

    /// Returns the payload, excluding all envelope headers.
    #[must_use]
    pub fn body(&self) -> &'packet [u8] {
        self.body
    }
}
