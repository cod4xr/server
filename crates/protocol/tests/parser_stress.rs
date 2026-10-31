// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic hostile-byte stress coverage; not a coverage-guided fuzzer.
#![allow(
    clippy::expect_used,
    reason = "Stress harness controls bounded buffers."
)]

use cod4x_protocol::compression;
use cod4x_protocol::{CommandLine, Direction, InfoLimit, InfoString, Packet};

fn next_random(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

#[test]
fn varied_hostile_packets_and_strings_have_bounded_transactional_behavior() {
    let mut state = 0x5eed_cafe;
    let mut bytes = [0; 1536];
    for case in 0..4096 {
        let length = usize::try_from(next_random(&mut state) % 1537).expect("Length budget");
        for byte in &mut bytes[..length] {
            *byte = next_random(&mut state).to_le_bytes()[0];
        }
        if length >= 4 {
            let marker: Option<u32> = match case % 6 {
                0 => Some(u32::MAX),
                1 => Some(0xffff_fffe),
                2 => Some(0xffff_fff0),
                3 => Some(1),
                4 => Some(0x8000_0001),
                _ => None,
            };
            if let Some(marker) = marker {
                bytes[..4].copy_from_slice(&marker.to_le_bytes());
            }
        }
        let input = &bytes[..length];
        for direction in [Direction::ClientToServer, Direction::ServerToClient] {
            if let Ok(packet) = Packet::parse(input, direction) {
                let mut encoded = [0xa5; 1600];
                assert_eq!(packet.encode(&mut encoded), Ok(length));
                assert_eq!(&encoded[..length], input);
                assert!(encoded[length..].iter().all(|&byte| byte == 0xa5));
            }
        }
        if let Ok(line) = CommandLine::parse(input) {
            assert!(
                line.tokens().count() <= length + 1,
                "Token iteration must terminate"
            );
            assert!(line.consumed_bytes() <= length);
        }
        if let Ok(info) = InfoString::parse(input, InfoLimit::Big) {
            assert!(info.pairs().count() <= length / 2);
            for pair in info.pairs() {
                assert!(!pair.key().is_empty());
                assert!(info.get(pair.key()).is_some());
            }
        }
        let mut decoded = [0xa5; 8192];
        match compression::decompress(input, &mut decoded) {
            Ok(length) => assert!(decoded[length..].iter().all(|&byte| byte == 0xa5)),
            Err(_) => assert!(decoded.iter().all(|&byte| byte == 0xa5)),
        }
    }
}

#[test]
fn every_two_byte_compressed_input_is_memory_safe_and_atomic_on_error() {
    for value in 0..=u16::MAX {
        let mut output = [0xa5; 32];
        match compression::decompress(&value.to_le_bytes(), &mut output) {
            Ok(length) => assert!(output[length..].iter().all(|&byte| byte == 0xa5)),
            Err(_) => assert_eq!(output, [0xa5; 32]),
        }
    }
}
