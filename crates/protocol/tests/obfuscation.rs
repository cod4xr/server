// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original XOR compatibility, inverse operation, and secret-free typed failures.
#![allow(
    clippy::expect_used,
    reason = "Reference fixture fields are validated test data."
)]

use cod4x_protocol::obfuscation::{self, ObfuscationError};
use cod4x_protocol::{MAX_COMMAND_BYTES, MAX_MESSAGE_BYTES};

fn decode_hex(hex: &str) -> Vec<u8> {
    if hex == "~" {
        return Vec::new();
    }
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16).expect("Hex byte")
        })
        .collect()
}

#[test]
fn both_directions_match_original_functions_and_are_involutions() {
    let mut count = 0;
    for line in include_str!("fixtures/obfuscation_vectors.txt").lines() {
        if !line.starts_with("xor ") {
            continue;
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        let command = decode_hex(fields[2]);
        let original = decode_hex(fields[3]);
        let expected = decode_hex(fields[4]);
        let mut actual = original.clone();
        for pass in 0..2 {
            match fields[1] {
                "0" => obfuscation::apply_server_payload(&mut actual, 0x1234_abcd, 0x101, &command),
                "1" => obfuscation::apply_client_payload(
                    &mut actual,
                    0x1234_abcd,
                    0x55,
                    0x1234,
                    &command,
                ),
                _ => unreachable!("Source fixture contains both audited directions"),
            }
            .expect("Bounded transform");
            assert_eq!(
                actual,
                if pass == 0 { &expected } else { &original }.as_slice()
            );
        }
        count += 1;
    }
    assert_eq!(count, 48);
}

#[test]
fn command_cycles_use_the_c_string_prefix_and_accept_zero_padded_storage() {
    let mut padded = vec![0; MAX_COMMAND_BYTES + 64];
    padded[..3].copy_from_slice(b"abc");
    let mut first = [1, 2, 3, 4, 5];
    let mut second = first;
    obfuscation::apply_server_payload(&mut first, 1, 2, &padded).expect("NUL prefix");
    obfuscation::apply_server_payload(&mut second, 1, 2, b"abc").expect("Unterminated slice");
    assert_eq!(first, second);
    let mut empty = [1, 2, 3, 4];
    let mut terminated_empty = empty;
    obfuscation::apply_client_payload(&mut empty, 1, 2, 3, &[]).expect("Empty command");
    obfuscation::apply_client_payload(&mut terminated_empty, 1, 2, 3, &[0; 1024])
        .expect("Empty C string");
    assert_eq!(empty, terminated_empty);
}

#[test]
fn limits_reject_before_mutating_the_payload() {
    let mut payload = [0xa5; 8];
    let command = vec![b'x'; MAX_COMMAND_BYTES + 1];
    assert_eq!(
        obfuscation::apply_server_payload(&mut payload, 1, 2, &command),
        Err(ObfuscationError::CommandTooLong)
    );
    assert_eq!(payload, [0xa5; 8]);
    let mut oversized = vec![0xa5; MAX_MESSAGE_BYTES + 1];
    assert_eq!(
        obfuscation::apply_client_payload(&mut oversized, 1, 2, 3, b"command"),
        Err(ObfuscationError::MessageTooLarge)
    );
    assert!(oversized.iter().all(|&byte| byte == 0xa5));
}

#[test]
fn empty_command_stale_tail_bytes_are_deliberately_excluded_from_key_material() {
    let line = include_str!("fixtures/obfuscation_vectors.txt")
        .lines()
        .find(|line| line.starts_with("stale-empty "))
        .expect("Original bug reproducer");
    let fields: Vec<_> = line.split_ascii_whitespace().collect();
    let stale_command = decode_hex(fields[1]);
    let mut actual = decode_hex(fields[2]);
    let mut canonical = actual.clone();
    let original_stale_result = decode_hex(fields[3]);
    obfuscation::apply_client_payload(&mut actual, 0x1234_abcd, 0x55, 0x1234, &stale_command)
        .expect("Bounded declared empty command");
    obfuscation::apply_client_payload(&mut canonical, 0x1234_abcd, 0x55, 0x1234, &[])
        .expect("Canonical empty command");
    assert_eq!(actual, canonical);
    assert_ne!(
        actual, original_stale_result,
        "Original stale-memory behavior must remain an explicit compatibility deviation"
    );
}
