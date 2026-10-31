// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};

/// One nonblocking UDP socket and its process-owned readiness registration.
/// Runtime drains receives until `WouldBlock` before waiting again. Poll readiness
/// may be spurious; only a successful receive establishes an actual datagram.
pub(crate) struct ReadySocket {
    socket: UdpSocket,
    poll: Poll,
    events: Events,
}

impl ReadySocket {
    pub(crate) fn bind(address: SocketAddr) -> io::Result<Self> {
        let mut socket = UdpSocket::bind(address)?;
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut socket, Token(0), Interest::READABLE)?;
        Ok(Self {
            socket,
            poll,
            events: Events::with_capacity(8),
        })
    }

    pub(crate) fn local_address(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub(crate) fn receive(&self, input: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(input)
    }

    pub(crate) fn send(&self, output: &[u8], destination: SocketAddr) -> io::Result<usize> {
        self.socket.send_to(output, destination)
    }

    pub(crate) fn wait(&mut self, timeout: Duration) -> io::Result<()> {
        // Only one readable socket is registered. Any wakeup triggers another
        // nonblocking receive; timeout never cancels an outstanding receive.
        self.poll.poll(&mut self.events, Some(timeout))
    }
}
