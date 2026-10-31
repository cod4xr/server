// SPDX-License-Identifier: AGPL-3.0-or-later
//! Differential coverage against generated fixtures from the pinned C source.
#![allow(
    clippy::expect_used,
    reason = "Test failures must identify the failed invariant."
)]

use cod4x_protocol::{MessageReader, MessageWriter};

#[derive(Debug)]
enum Operation {
    Bits { value: u32, width: u8 },
    Byte(u8),
    Short(i16),
    Long(i32),
    Wide(i64),
    Float(u32),
}

fn next_value(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

fn operations(seed: u32) -> Vec<Operation> {
    let mut state = seed;
    let mut operations = Vec::new();
    for _ in 0..96 {
        let kind = next_value(&mut state) % 6;
        let raw = next_value(&mut state);
        let bytes = raw.to_le_bytes();
        let operation = match kind {
            0 => Operation::Bits {
                value: raw,
                width: u8::try_from(next_value(&mut state) % 33).expect("Width is at most 32"),
            },
            1 => Operation::Byte(bytes[0]),
            2 => Operation::Short(i16::from_le_bytes([bytes[0], bytes[1]])),
            3 => Operation::Long(i32::from_le_bytes(bytes)),
            4 => {
                let wide = (u64::from(next_value(&mut state)) << 32) | u64::from(raw);
                Operation::Wide(i64::from_le_bytes(wide.to_le_bytes()))
            }
            5 => Operation::Float(raw & 0xff7f_ffff),
            _ => unreachable!("Remainder modulo six is in 0..6"),
        };
        operations.push(operation);
    }
    operations
}

fn encode(writer: &mut MessageWriter<'_>, operation: &Operation) {
    let result = match *operation {
        Operation::Bits { value, width } => writer.write_bits(value, width),
        Operation::Byte(value) => writer.write_u8(value),
        Operation::Short(value) => writer.write_i16(value),
        Operation::Long(value) => writer.write_i32(value),
        Operation::Wide(value) => writer.write_i64(value),
        Operation::Float(bits) => writer.write_f32(f32::from_bits(bits)),
    };
    result.expect("The fixture writer has sufficient capacity");
}

fn check_decoding(reader: &mut MessageReader<'_>, operation: &Operation) {
    match *operation {
        Operation::Bits { value, width } => {
            let mask = match width {
                0 => 0,
                32 => u32::MAX,
                _ => (1_u32 << width) - 1,
            };
            assert_eq!(
                reader.read_bits(width).expect("Complete bit field"),
                value & mask
            );
        }
        Operation::Byte(value) => assert_eq!(reader.read_u8().expect("Complete byte"), value),
        Operation::Short(value) => assert_eq!(reader.read_i16().expect("Complete i16"), value),
        Operation::Long(value) => assert_eq!(reader.read_i32().expect("Complete i32"), value),
        Operation::Wide(value) => assert_eq!(reader.read_i64().expect("Complete i64"), value),
        Operation::Float(bits) => {
            assert_eq!(reader.read_f32().expect("Complete f32").to_bits(), bits);
        }
    }
}

fn decode_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "Every fixture byte has two hex digits");
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("Hex fixture is ASCII");
            u8::from_str_radix(text, 16).expect("Fixture byte is hexadecimal")
        })
        .collect()
}

#[test]
fn encoding_and_decoding_match_upstream_c_for_every_split_boundary() {
    let mut count = 0;
    let rows = include_str!("fixtures/message_vectors.txt").lines();
    for row in rows {
        if row.starts_with('#') {
            continue;
        }
        let (seed, hex) = row.split_once(' ').expect("Fixture has seed and bytes");
        let seed: u32 = seed.parse().expect("Seed is an unsigned integer");
        let expected = decode_hex(hex);
        let operations = operations(seed);
        let mut storage = [0xa5; 4096];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Bounded storage");
        for operation in &operations {
            encode(&mut writer, operation);
        }
        assert_eq!(
            writer.as_bytes(),
            expected,
            "C wire mismatch for seed {seed}"
        );
        for boundary in 0..=expected.len() {
            let (first, second) = expected.split_at(boundary);
            let mut reader = MessageReader::try_new_split(first, second).expect("Bounded fixture");
            for operation in &operations {
                check_decoding(&mut reader, operation);
            }
            assert_eq!(
                reader.remaining_bytes(),
                0,
                "Unconsumed bytes for seed {seed}"
            );
        }
        count += 1;
    }
    assert_eq!(
        count, 64,
        "All generated reference vectors must be exercised"
    );
}
