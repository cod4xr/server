// SPDX-License-Identifier: AGPL-3.0-or-later
use std::net::SocketAddr;
use std::time::Duration;

use cod4x_server_core::{
    DiscoveryService, ENDPOINT_COOKIE_KEY_BYTES, EndpointCookie, HandshakeProbe, ProbeOutcome,
    QueryOutcome,
};
use zeroize::Zeroizing;

use crate::app_error::AppError;
use crate::arguments::Mode;
use crate::counters::Counters;

pub(crate) enum RuntimeService {
    QueryOnly(DiscoveryService),
    Probe(HandshakeProbe),
}

impl RuntimeService {
    pub(crate) fn for_mode(mode: &Mode) -> Result<Self, AppError> {
        match mode {
            Mode::QueryOnly => Ok(Self::QueryOnly(DiscoveryService::default())),
            Mode::HandshakeProbe => {
                let mut key = Zeroizing::new([0; ENDPOINT_COOKIE_KEY_BYTES]);
                getrandom::fill(&mut *key).map_err(AppError::Entropy)?;
                let cookies = EndpointCookie::try_new(&key[..]).map_err(AppError::Cookie)?;
                Ok(Self::Probe(HandshakeProbe::new(cookies)))
            }
            Mode::VerifyIwd(_) | Mode::InspectFastFile(_) | Mode::InspectZonePrefix(_) => {
                Err(AppError::Arguments("content inspection is offline"))
            }
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::QueryOnly(_) => "query_only",
            Self::Probe(_) => "handshake_probe",
        }
    }

    pub(crate) fn handle(
        &mut self,
        source: SocketAddr,
        input: &[u8],
        output: &mut [u8],
        now: Duration,
        counters: &mut Counters,
    ) -> Option<usize> {
        match self {
            Self::QueryOnly(service) => {
                let outcome = service.handle(source.ip(), input, output, now);
                counters.record(outcome);
                match outcome {
                    QueryOutcome::Reply { length, .. } => Some(length),
                    _ => None,
                }
            }
            Self::Probe(service) => {
                let outcome = service.handle(source, input, output, now);
                counters.record_probe(outcome);
                match outcome {
                    ProbeOutcome::Reply { length, .. } => Some(length),
                    _ => None,
                }
            }
        }
    }
}
