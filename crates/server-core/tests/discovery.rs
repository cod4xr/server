// SPDX-License-Identifier: AGPL-3.0-or-later
//! Query byte contracts and state-budget isolation without sockets or wall clocks.
#![allow(
    clippy::expect_used,
    reason = "Tests construct known-valid packet fixtures."
)]

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use cod4x_protocol::{CommandLine, Direction, InfoLimit, InfoString, Packet};
use cod4x_server_core::{DiscoveryService, DropReason, QueryKind, QueryOutcome};

fn address(last: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(192, 0, 2, last))
}

fn query(command: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0; command.len() + 4];
    Packet::Connectionless(command)
        .encode(&mut bytes)
        .expect("Valid query");
    bytes
}

#[test]
fn discovery_echoes_challenges_and_advertises_an_explicit_zero_slot_development_server() {
    let mut service = DiscoveryService::default();
    for (command, kind, prefix) in [
        (
            b"getinfo a%b".as_slice(),
            QueryKind::Info,
            b"infoResponse\n".as_slice(),
        ),
        (b"GETSTATUS \"a%b\"", QueryKind::Status, b"statusResponse\n"),
    ] {
        let mut output = [0xa5; 1400];
        let result = service.handle(address(1), &query(command), &mut output, Duration::ZERO);
        let QueryOutcome::Reply {
            kind: actual,
            length,
        } = result
        else {
            unreachable!("A valid development query must receive a reply");
        };
        assert_eq!(actual, kind);
        let packet = Packet::parse(&output[..length], Direction::ServerToClient)
            .expect("Valid output envelope");
        let Packet::Connectionless(body) = packet else {
            unreachable!("Query replies are connectionless");
        };
        assert!(body.starts_with(prefix));
        let line = CommandLine::parse(&body[prefix.len()..]).expect("Info line");
        let info = InfoString::parse(line.as_bytes(), InfoLimit::Normal).expect("Valid info");
        assert_eq!(info.get(b"challenge"), Some(b"a.b".as_slice()));
        assert_eq!(info.get(b"sv_maxclients"), Some(b"0".as_slice()));
        assert_eq!(info.get(b"clients"), Some(b"0".as_slice()));
        assert_eq!(info.get(b"protocol"), Some(b"6".as_slice()));
        assert!(
            info.get(b"hostname")
                .is_some_and(|name| name.ends_with(b"development)"))
        );
        assert_eq!(output[length], 0xa5);
        if kind == QueryKind::Status {
            assert_eq!(body.last(), Some(&b'\n'));
        }
    }
}

#[test]
fn invalid_or_unsupported_packets_never_mutate_output_or_consume_query_budget() {
    let mut service = DiscoveryService::default();
    for input in [
        vec![1],
        query(b"rcon secret command"),
        query(b"getinfo bad;value"),
        query(&[b"getinfo ".as_slice(), &[b'x'; 129]].concat()),
    ] {
        let mut output = [0xa5; 1400];
        assert!(matches!(
            service.handle(address(1), &input, &mut output, Duration::ZERO),
            QueryOutcome::Dropped(_)
        ));
        assert_eq!(output, [0xa5; 1400]);
    }
    for _ in 0..4 {
        assert!(matches!(
            service.handle(
                address(1),
                &query(b"getinfo"),
                &mut [0; 1400],
                Duration::ZERO
            ),
            QueryOutcome::Reply { .. }
        ));
    }
}

#[test]
fn per_ip_budget_normalizes_mapped_ipv4_and_refills_on_monotonic_time() {
    let mut service = DiscoveryService::default();
    let input = query(b"getinfo");
    let mut output = [0; 1400];
    for _ in 0..4 {
        assert!(matches!(
            service.handle(address(1), &input, &mut output, Duration::ZERO),
            QueryOutcome::Reply { .. }
        ));
    }
    let mapped = IpAddr::V6(Ipv4Addr::new(192, 0, 2, 1).to_ipv6_mapped());
    assert_eq!(
        service.handle(mapped, &input, &mut output, Duration::ZERO),
        QueryOutcome::Dropped(DropReason::RateLimited)
    );
    assert!(matches!(
        service.handle(address(2), &input, &mut output, Duration::ZERO),
        QueryOutcome::Reply { .. }
    ));
    assert!(matches!(
        service.handle(address(1), &input, &mut output, Duration::from_millis(250)),
        QueryOutcome::Reply { .. }
    ));
    assert_eq!(
        service.handle(address(2), &input, &mut output, Duration::ZERO),
        QueryOutcome::Dropped(DropReason::RateLimited)
    );
}

#[test]
fn global_reply_budget_prevents_amplification_across_many_source_addresses() {
    let mut service = DiscoveryService::default();
    let input = query(b"getstatus");
    let mut output = [0; 1400];
    for last in 0..100 {
        assert!(matches!(
            service.handle(address(last), &input, &mut output, Duration::ZERO),
            QueryOutcome::Reply { .. }
        ));
    }
    assert_eq!(
        service.handle(address(101), &input, &mut output, Duration::ZERO),
        QueryOutcome::Dropped(DropReason::RateLimited)
    );
    assert!(matches!(
        service.handle(address(101), &input, &mut output, Duration::from_millis(10)),
        QueryOutcome::Reply { .. }
    ));
}

#[test]
fn insufficient_reply_capacity_is_an_atomic_drop() {
    let mut service = DiscoveryService::default();
    let mut output = [0xa5; 8];
    assert_eq!(
        service.handle(address(1), &query(b"getinfo"), &mut output, Duration::ZERO),
        QueryOutcome::Dropped(DropReason::OutputCapacity)
    );
    assert_eq!(output, [0xa5; 8]);
}
