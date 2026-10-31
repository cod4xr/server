// SPDX-License-Identifier: AGPL-3.0-or-later
use super::ChannelPacket;

/// Canonical maximum channel datagram size from upstream `MAX_PACKETLEN`.
/// Connectionless responses can be larger; socket-level limits apply separately.
pub const MAX_PACKET_BYTES: usize = 1400;

/// A full channel fragment carries 1300 bytes; a shorter fragment terminates it.
pub const FRAGMENT_BYTES: usize = MAX_PACKET_BYTES - 100;

/// Direction of an incoming datagram. Ordinary client packets include a qport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// An incoming packet sent by a client.
    ClientToServer,
    /// An incoming packet sent by a server.
    ServerToClient,
}

/// A validated channel route, including the qport only where required.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
    /// Client-to-server traffic includes this logical NAT routing identifier.
    ToServer {
        /// Logical client port; distinct from the observed UDP source port.
        qport: u16,
    },
    /// Ordinary server-to-client traffic does not include a qport.
    ToClient,
}

/// A borrowed datagram envelope. Bodies remain untrusted until separately parsed.
///
/// The two special `CoD4X` transports must be classified before the fragment bit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Packet<'packet> {
    /// Four `0xff` bytes followed by a binary or textual out-of-band body.
    Connectionless(&'packet [u8]),
    /// Marker `0xfffffffe`; update-proxy body is intentionally uninterpreted.
    UpdateProxy(&'packet [u8]),
    /// Marker `0xfffffff0`; this transport includes a qport in both directions.
    Reliable {
        /// Logical routing identifier.
        qport: u16,
        /// Reliable-transport message requiring its own validation.
        body: &'packet [u8],
    },
    /// A normal channel packet, possibly a fragment.
    Channel(ChannelPacket<'packet>),
}
