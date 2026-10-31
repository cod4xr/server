// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use cod4x_server_core::{DropReason, ProbeOutcome, ProbeReply, QueryOutcome};

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
    probe_challenges: u64,
    probe_verified_rejections: u64,
}

impl Counters {
    pub(crate) fn record(&mut self, outcome: QueryOutcome) {
        match outcome {
            QueryOutcome::Reply { .. } => (),
            QueryOutcome::Dropped(reason) => self.record_drop(reason),
            _ => self.unclassified = self.unclassified.saturating_add(1),
        }
    }

    pub(crate) fn record_probe(&mut self, outcome: ProbeOutcome) {
        let counter = match outcome {
            ProbeOutcome::Reply {
                kind: ProbeReply::Challenge,
                ..
            } => &mut self.probe_challenges,
            ProbeOutcome::Reply {
                kind: ProbeReply::Unavailable,
                ..
            } => &mut self.probe_verified_rejections,
            ProbeOutcome::Dropped(reason) => {
                self.record_drop(reason);
                return;
            }
            _ => &mut self.unclassified,
        };
        *counter = counter.saturating_add(1);
    }

    fn record_drop(&mut self, reason: DropReason) {
        let counter = match reason {
            DropReason::Malformed => &mut self.malformed,
            DropReason::Unsupported => &mut self.unsupported,
            DropReason::InvalidChallenge => &mut self.invalid_challenge,
            DropReason::RateLimited => &mut self.rate_limited,
            DropReason::OutputCapacity => &mut self.output_capacity,
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
            probe_challenges = self.probe_challenges,
            probe_verified_rejections = self.probe_verified_rejections,
            dropped_logs,
            "server counters"
        );
    }
}
