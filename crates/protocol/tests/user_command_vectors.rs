// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent C command decoding at every storage split and truncation boundary.
#![allow(
    clippy::expect_used,
    reason = "Tests decode pinned original C fixtures."
)]

use std::fmt::Debug;
use std::str::FromStr;

use cod4x_protocol::MessageReader;
use cod4x_protocol::user_command::{self, LegacyUserCommand};

struct Vector {
    key: u32,
    bytes: Vec<u8>,
    baseline: LegacyUserCommand,
    expected: LegacyUserCommand,
    used: usize,
    tail_bits: u32,
    tail_byte: u8,
}

fn value<T: FromStr>(text: &str) -> T
where
    T::Err: Debug,
{
    text.parse().expect("Fixture field")
}

fn command(text: &str) -> LegacyUserCommand {
    let fields: Vec<&str> = text.split(',').collect();
    assert_eq!(fields.len(), 13);
    LegacyUserCommand {
        server_time: value(fields[0]),
        buttons: value(fields[1]),
        angles: [value(fields[2]), value(fields[3]), value(fields[4])],
        weapon: value(fields[5]),
        offhand: value(fields[6]),
        forward_move: value(fields[7]),
        right_move: value(fields[8]),
        melee_charge_yaw: f32::from_bits(u32::from_str_radix(fields[9], 16).expect("Yaw bits")),
        melee_charge_distance: value(fields[10]),
        selected_location: [value(fields[11]), value(fields[12])],
    }
}

fn vectors() -> Vec<Vector> {
    include_str!("fixtures/user_command_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
        .map(|row| {
            let fields: Vec<&str> = row.split_whitespace().collect();
            assert_eq!(fields.len(), 7);
            let bytes = fields[1]
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| {
                    u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16)
                        .expect("Byte")
                })
                .collect();
            Vector {
                key: u32::from_str_radix(fields[0], 16).expect("Key"),
                bytes,
                baseline: command(fields[2]),
                expected: command(fields[3]),
                used: value(fields[4]),
                tail_bits: value(fields[5]),
                tail_byte: value(fields[6]),
            }
        })
        .collect()
}

#[test]
fn both_delta_branches_conditional_fields_and_cursor_tails_match_original_c_at_every_split() {
    let vectors = vectors();
    assert_eq!(vectors.len(), 512);
    for (index, vector) in vectors.iter().enumerate() {
        for split in 0..=vector.bytes.len() {
            let mut reader =
                MessageReader::try_new_split(&vector.bytes[..split], &vector.bytes[split..])
                    .expect("Split reader");
            let decoded = user_command::read_delta(&mut reader, vector.key, &vector.baseline)
                .expect("Complete command");
            assert_eq!(decoded, vector.expected, "sample {index}, split {split}");
            assert_eq!(
                decoded.melee_charge_yaw.to_bits(),
                vector.expected.melee_charge_yaw.to_bits()
            );
            assert_eq!(vector.bytes.len() - reader.remaining_bytes(), vector.used);
            assert_eq!(reader.read_bits(3).expect("Cached bits"), vector.tail_bits);
            assert_eq!(reader.read_u8().expect("Following byte"), vector.tail_byte);
        }
    }
}

#[test]
fn every_truncated_original_command_preserves_both_reader_cursors() {
    for vector in vectors() {
        for end in 0..vector.used {
            let mut reader =
                MessageReader::try_new(&vector.bytes[..end]).expect("Truncated reader");
            let mut control = reader.clone();
            assert!(user_command::read_delta(&mut reader, vector.key, &vector.baseline).is_err());
            assert_eq!(reader.remaining_bytes(), end);
            assert_eq!(reader.read_bits(3), control.read_bits(3));
            assert_eq!(reader.read_u8(), control.read_u8());
        }
    }
}
