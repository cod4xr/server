// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use cod4x_server_core::{DropReason, QueryOutcome};

#[derive(Debug, Default)]
pub(crate) struct Counters {
    pub(crate) received: u64,
    pub(crate) replied: u64,
    pub(crate) send_errors: u64,
    malformed: u64,
    unsupported: u64,
    invalid_challenge: u64,
    rate_limited: u64,
    output_capacity: u64,
    unclassified: u64,
}

impl Counters {
    pub(crate) fn record(&mut self, outcome: QueryOutcome) {
        let counter = match outcome {
            QueryOutcome::Reply { .. } => return,
            QueryOutcome::Dropped(reason) => match reason {
                DropReason::Malformed => &mut self.malformed,
                DropReason::Unsupported => &mut self.unsupported,
                DropReason::InvalidChallenge => &mut self.invalid_challenge,
                DropReason::RateLimited => &mut self.rate_limited,
                DropReason::OutputCapacity => &mut self.output_capacity,
                _ => &mut self.unclassified,
            },
            _ => &mut self.unclassified,
        };
        *counter = counter.saturating_add(1);
    }

    pub(crate) fn report(&self, uptime: Duration, dropped_logs: usize) {
        let uptime_ms = u64::try_from(uptime.as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            event = "server.metrics",
            uptime_ms,
            received = self.received,
            replied = self.replied,
            malformed = self.malformed,
            unsupported = self.unsupported,
            invalid_challenge = self.invalid_challenge,
            rate_limited = self.rate_limited,
            output_capacity = self.output_capacity,
            send_errors = self.send_errors,
            unclassified = self.unclassified,
            dropped_logs,
            "server counters"
        );
    }
}
