// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use cod4x_observability::LogRuntime;
use cod4x_protocol::MAX_PACKET_BYTES;

use crate::arguments::Arguments;
use crate::counters::Counters;
use crate::service::RuntimeService;

pub(crate) struct Runtime {
    socket: UdpSocket,
    service: RuntimeService,
    counters: Counters,
}

impl Runtime {
    pub(crate) fn bind(address: SocketAddr, service: RuntimeService) -> io::Result<Self> {
        let socket = UdpSocket::bind(address)?;
        socket.set_read_timeout(Some(Duration::from_millis(50)))?;
        Ok(Self {
            socket,
            service,
            counters: Counters::default(),
        })
    }

    pub(crate) fn local_address(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub(crate) fn run(
        &mut self,
        options: &Arguments,
        stop: &AtomicBool,
        logs: &LogRuntime,
    ) -> io::Result<&'static str> {
        let start = Instant::now();
        let mut next_metrics = Duration::from_secs(10);
        // A full conventional UDP payload fits, so oversized input is never
        // truncated into a smaller apparently valid command by the socket API.
        let mut input = vec![0; 65_536];
        let mut output = [0; MAX_PACKET_BYTES];
        let result = loop {
            let elapsed = start.elapsed();
            if stop.load(Ordering::Relaxed) {
                break Ok("signal");
            }
            if options.duration.is_some_and(|duration| elapsed >= duration) {
                break Ok("duration");
            }
            if options
                .max_datagrams
                .is_some_and(|limit| self.counters.received >= limit)
            {
                break Ok("datagram_limit");
            }
            if elapsed >= next_metrics {
                self.counters.report(elapsed, logs.dropped_records());
                next_metrics = elapsed.saturating_add(Duration::from_secs(10));
            }
            let (length, source) = match self.socket.recv_from(&mut input) {
                Ok(received) => received,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                            | io::ErrorKind::ConnectionReset
                    ) =>
                {
                    continue;
                }
                Err(error) => break Err(error),
            };
            self.counters.received = self.counters.received.saturating_add(1);
            let received_at = start.elapsed();
            if let Some(length) = self.service.handle(
                source,
                &input[..length],
                &mut output,
                received_at,
                &mut self.counters,
            ) {
                match self.socket.send_to(&output[..length], source) {
                    Ok(sent) if sent == length => {
                        self.counters.replied = self.counters.replied.saturating_add(1);
                    }
                    Ok(_) | Err(_) => {
                        self.counters.send_errors = self.counters.send_errors.saturating_add(1);
                    }
                }
            }
        };
        self.counters
            .report(start.elapsed(), logs.dropped_records());
        result
    }
}
