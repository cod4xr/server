// SPDX-License-Identifier: AGPL-3.0-or-later
//! Client-directed challenge codecs share the executed original response vectors.
#![allow(
    clippy::expect_used,
    reason = "Tests replay finite original and independent wire vectors."
)]
use cod4x_protocol::connection::{ChallengeError, ChallengeRequest, ChallengeResponse};
use cod4x_protocol::{CommandLine, Direction, Packet};

#[test]
fn all_original_response_datagrams_decode_signed_fields_without_changing_encoder_bytes() {
    let mut count = 0;
    for row in include_str!("fixtures/wire_vectors.txt")
        .lines()
        .filter_map(|row| row.strip_prefix("challenge "))
    {
        let fields: Vec<&str> = row.split_whitespace().collect();
        let bytes: Vec<u8> = fields[2]
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("Hex"), 16).expect("Byte")
            })
            .collect();
        let reply = ChallengeResponse::parse_packet(&bytes).expect("Original response");
        assert_eq!(reply.server_challenge, fields[0].parse().expect("Cookie"));
        assert_eq!(reply.client_challenge, fields[1].parse().expect("Nonce"));
        let mut encoded = [0xaa; 128];
        let length = reply.encode(&mut encoded).expect("Original encoder");
        assert_eq!(&encoded[..length], bytes);
        assert_eq!(format!("{reply:?}"), "ChallengeResponse { .. }");
        count += 1;
    }
    assert_eq!(count, 40);
}

#[test]
fn nonce_only_request_bytes_and_every_insufficient_capacity_are_transactional() {
    for nonce in [i32::MIN, -1, 0, 1, i32::MAX] {
        let request = ChallengeRequest {
            client_challenge: nonce,
        };
        let mut bytes = [0xa5; 64];
        let length = request.encode(&mut bytes).expect("Request");
        let expected = format!("getchallenge {nonce}");
        assert_eq!(&bytes[..4], &[0xff; 4]);
        assert_eq!(&bytes[4..length], expected.as_bytes());
        assert_eq!(format!("{request:?}"), "ChallengeRequest { .. }");
        assert!(bytes[length..].iter().all(|&byte| byte == 0xa5));
        assert!(matches!(
            Packet::parse(&bytes[..length], Direction::ClientToServer),
            Ok(Packet::Connectionless(_))
        ));
        for capacity in 0..length {
            let mut output = [0xa5; 64];
            assert!(request.encode(&mut output[..capacity]).is_err());
            assert_eq!(output, [0xa5; 64]);
        }
    }
}

#[test]
fn every_truncated_fixed_profile_response_rejects() {
    let packet = b"\xff\xff\xff\xffchallengeResponse -2147483648 2147483647 0 xproto 18";
    for length in 0..packet.len() {
        assert!(
            ChallengeResponse::parse_packet(&packet[..length]).is_err(),
            "prefix {length}"
        );
    }
    assert!(ChallengeResponse::parse_packet(packet).is_ok());
}

#[test]
fn terminal_delimiters_are_distinct_from_unparsed_trailing_data() {
    let mut packet = b"\xff\xff\xff\xffchallengeResponse 1 2 0 xproto 18".to_vec();
    for delimiter in [b'\n', 0] {
        packet.push(delimiter);
        assert!(ChallengeResponse::parse_packet(&packet).is_ok());
        packet.push(b'x');
        assert_eq!(
            ChallengeResponse::parse_packet(&packet).expect_err("Unparsed suffix"),
            ChallengeError::TrailingData
        );
        packet.truncate(packet.len() - 2);
    }
}

#[test]
fn malformed_numeric_fields_and_each_non_digit_byte_reject_without_panics() {
    for value in ["", "-", "+1", "1x", "2147483648", "-2147483649", "1.0"] {
        let text = format!("challengeResponse \"{value}\" 2 0 xproto 18");
        assert_eq!(
            ChallengeResponse::parse(CommandLine::parse(text.as_bytes()).expect("Line"))
                .expect_err("Bad integer"),
            ChallengeError::InvalidInteger
        );
    }
    for byte in 33_u8..=u8::MAX {
        if byte.is_ascii_digit() || matches!(byte, b'"' | b'/' | b'\x7f') {
            continue;
        }
        let mut packet = b"\xff\xff\xff\xffchallengeResponse ".to_vec();
        packet.push(byte);
        packet.extend_from_slice(b" 2 0 xproto 18");
        assert!(
            ChallengeResponse::parse_packet(&packet).is_err(),
            "byte {byte}"
        );
    }
}

#[test]
fn unsupported_suffixes_do_not_silently_become_legacy_or_negotiated_protocols() {
    for text in [
        "challengeResponse 1",
        "challengeResponse 1 2",
        "challengeResponse 1 2 0 xproto 21",
        "challengeResponse 1 2 1 xproto 18",
        "challengeResponse 1 2 0 other 18",
        "challengeResponse 1 2 0 xproto 18 extra",
        "connectResponse 1 2 0 xproto 18",
    ] {
        assert!(
            ChallengeResponse::parse(CommandLine::parse(text.as_bytes()).expect("Line")).is_err()
        );
    }
    let text = b"CHALLENGERESPONSE 1 2 0 XPROTO 18";
    assert!(ChallengeResponse::parse(CommandLine::parse(text).expect("Line")).is_ok());
}
