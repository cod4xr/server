// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent HMAC fixtures, endpoint binding, fixed tag length, and redaction.
#![allow(
    clippy::expect_used,
    reason = "Tests use exclusively synthetic cookie keys/endpoints."
)]

use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6};

use cod4x_server_core::{CookieKeyError, EndpointCookie};

#[test]
fn cookies_match_independent_python_hmac_for_ipv4_ipv6_mapped_addresses_and_scope() {
    let mut count = 0;
    for row in include_str!("fixtures/cookie_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<&str> = row.split_whitespace().collect();
        assert_eq!(fields.len(), 5);
        let key: Vec<u8> = fields[0]
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16).expect("Byte")
            })
            .collect();
        let ip: IpAddr = fields[1].parse().expect("IP");
        let port = fields[2].parse().expect("Port");
        let scope = fields[3].parse().expect("Scope");
        let endpoint = match ip {
            IpAddr::V4(ip) => SocketAddr::from((ip, port)),
            IpAddr::V6(ip) => SocketAddr::V6(SocketAddrV6::new(ip, port, 0, scope)),
        };
        let expected: i32 = fields[4].parse().expect("Signed challenge");
        let cookies = EndpointCookie::try_new(&key).expect("Provider");
        assert_eq!(cookies.issue(endpoint), expected);
        assert!(cookies.verify(endpoint, expected));
        for bit in 0..32 {
            let altered = i32::from_le_bytes(
                (u32::from_le_bytes(expected.to_le_bytes()) ^ (1 << bit)).to_le_bytes(),
            );
            assert!(!cookies.verify(endpoint, altered));
        }
        count += 1;
    }
    assert_eq!(count, 24);
}

#[test]
fn changed_address_port_scope_or_key_is_rejected_but_flow_information_is_not_identity() {
    let cookies = EndpointCookie::try_new(&[0xa5; 32]).expect("Provider");
    let ip = Ipv6Addr::LOCALHOST;
    let endpoint = SocketAddr::V6(SocketAddrV6::new(ip, 28960, 0, 4));
    let challenge = cookies.issue(endpoint);
    for other in [
        SocketAddr::V6(SocketAddrV6::new(ip, 28961, 0, 4)),
        SocketAddr::V6(SocketAddrV6::new(ip, 28960, 0, 5)),
        "[::2]:28960".parse().expect("Other IP"),
    ] {
        assert!(!cookies.verify(other, challenge));
    }
    assert!(cookies.verify(
        SocketAddr::V6(SocketAddrV6::new(ip, 28960, 12345, 4)),
        challenge
    ));
    let rotated = EndpointCookie::try_new(&[0x5a; 32]).expect("Rotated provider");
    assert!(!rotated.verify(endpoint, challenge));
}

#[test]
fn key_policy_and_debug_output_never_expose_secret_material() {
    for length in [0, 1, 31, 33, 64] {
        assert_eq!(
            EndpointCookie::try_new(&vec![0xa5; length]).err(),
            Some(CookieKeyError)
        );
    }
    let key: &[u8; 32] = b"synthetic-cookie-key-for-tests!!";
    let provider = EndpointCookie::try_new(key).expect("Provider");
    assert_eq!(format!("{provider:?}"), "EndpointCookie { .. }");
    assert!(!CookieKeyError.to_string().contains("synthetic"));
}
