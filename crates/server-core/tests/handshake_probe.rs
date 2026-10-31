// SPDX-License-Identifier: AGPL-3.0-or-later
//! Stateless legacy challenge exchange and explicit non-admission behavior.
#![allow(
    clippy::expect_used,
    reason = "Tests own synthetic cookies and complete packets."
)]

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use cod4x_protocol::{Direction, Packet};
use cod4x_server_core::{DropReason, EndpointCookie, HandshakeProbe, ProbeOutcome, ProbeReply};

fn probe() -> HandshakeProbe {
    HandshakeProbe::new(EndpointCookie::try_new(&[0xa5; 32]).expect("Synthetic process key"))
}

fn packet(body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0; body.len() + 4];
    Packet::Connectionless(body)
        .encode(&mut bytes)
        .expect("Datagram");
    bytes
}

#[test]
fn challenges_echo_legacy_nonces_without_allocating_a_client_or_advertising_admission()
-> Result<(), String> {
    let endpoint: SocketAddr = "127.0.0.1:40000".parse().expect("Endpoint");
    let cookies = EndpointCookie::try_new(&[0xa5; 32]).expect("Provider");
    let expected_cookie = cookies.issue(endpoint);
    for (request, expected_nonce) in [
        (b"getchallenge".as_slice(), 0),
        (b"GETCHALLENGE 0 \"synthetic-guid\"", 0),
        (b"getchallenge -2147483648", i32::MIN),
        (b"getchallenge 2147483647", i32::MAX),
    ] {
        let mut probe = probe();
        let mut output = [0xa5; 1024];
        let ProbeOutcome::Reply { kind, length } =
            probe.handle(endpoint, &packet(request), &mut output, Duration::ZERO)
        else {
            return Err("Expected challenge reply".into());
        };
        assert_eq!(kind, ProbeReply::Challenge);
        let expected = format!("challengeResponse {expected_cookie} {expected_nonce} 0 xproto 18");
        assert_eq!(
            Packet::parse(&output[..length], Direction::ServerToClient).expect("Reply"),
            Packet::Connectionless(expected.as_bytes())
        );
        assert_eq!(format!("{probe:?}"), "HandshakeProbe { .. }");
    }
    Ok(())
}

#[test]
fn verified_connects_are_honestly_refused_and_cookies_bind_the_observed_endpoint()
-> Result<(), String> {
    let endpoint: SocketAddr = "127.0.0.1:40000".parse().expect("Endpoint");
    let cookies = EndpointCookie::try_new(&[0xa5; 32]).expect("Provider");
    let cookie = cookies.issue(endpoint);
    let mut probe = HandshakeProbe::new(cookies);
    let body = format!(
        "connect \"\\protocol\\21\\challenge\\{cookie}\\qport\\-1\\password\\synthetic-secret\""
    );
    let mut output = [0xa5; 1024];
    let other: SocketAddr = "127.0.0.1:40001".parse().expect("Other port");
    assert_eq!(
        probe.handle(other, &packet(body.as_bytes()), &mut output, Duration::ZERO),
        ProbeOutcome::Dropped(DropReason::InvalidChallenge)
    );
    assert_eq!(output, [0xa5; 1024]);
    let ProbeOutcome::Reply { kind, length } = probe.handle(
        endpoint,
        &packet(body.as_bytes()),
        &mut output,
        Duration::ZERO,
    ) else {
        return Err("Expected refusal".into());
    };
    assert_eq!(kind, ProbeReply::Unavailable);
    let reply = Packet::parse(&output[..length], Direction::ServerToClient).expect("Reply");
    assert!(matches!(reply, Packet::Connectionless(body) if body.starts_with(b"error\n")));
    assert!(
        !output[..length]
            .windows(15)
            .any(|part| part == b"connectResponse")
    );
    Ok(())
}

#[test]
fn malformed_or_unsupported_inputs_do_not_consume_reply_budgets_or_mutate_output() {
    let mut probe = probe();
    let endpoint: SocketAddr = "127.0.0.1:40000".parse().expect("Endpoint");
    let mut output = [0xa5; 1024];
    for body in [
        b"getinfo x".as_slice(),
        b"rcon synthetic-secret",
        b"getchallenge invalid",
        b"connect \"\\protocol\\21\"",
    ] {
        assert!(matches!(
            probe.handle(endpoint, &packet(body), &mut output, Duration::ZERO),
            ProbeOutcome::Dropped(_)
        ));
        assert_eq!(output, [0xa5; 1024]);
    }
    for _ in 0..4 {
        assert!(matches!(
            probe.handle(
                endpoint,
                &packet(b"getchallenge 0"),
                &mut output,
                Duration::ZERO
            ),
            ProbeOutcome::Reply { .. }
        ));
    }
    output.fill(0xa5);
    assert_eq!(
        probe.handle(
            endpoint,
            &packet(b"getchallenge 0"),
            &mut output,
            Duration::ZERO
        ),
        ProbeOutcome::Dropped(DropReason::RateLimited)
    );
    assert_eq!(output, [0xa5; 1024]);
}

#[test]
fn global_limits_clock_regression_and_output_capacity_have_explicit_atomic_drops() {
    let input = packet(b"getchallenge 0");
    let mut probe = probe();
    let mut output = [0xa5; 1024];
    for suffix in 0..100_u8 {
        let endpoint = SocketAddr::from((Ipv4Addr::new(198, 51, 100, suffix), 40000));
        assert!(matches!(
            probe.handle(endpoint, &input, &mut output, Duration::from_secs(1)),
            ProbeOutcome::Reply { .. }
        ));
    }
    let endpoint: SocketAddr = "192.0.2.1:40000".parse().expect("Endpoint");
    output.fill(0xa5);
    for time in [Duration::from_secs(1), Duration::ZERO] {
        assert_eq!(
            probe.handle(endpoint, &input, &mut output, time),
            ProbeOutcome::Dropped(DropReason::RateLimited)
        );
        assert_eq!(output, [0xa5; 1024]);
    }
    assert_eq!(
        probe.handle(endpoint, &input, &mut [], Duration::from_secs(2)),
        ProbeOutcome::Dropped(DropReason::OutputCapacity)
    );
}
