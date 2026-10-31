// SPDX-License-Identifier: AGPL-3.0-or-later
//! Differential fixtures from original channel, info-string, and token functions.
#![allow(
    clippy::expect_used,
    reason = "Reference fixtures must be internally well formed."
)]

use cod4x_protocol::{
    ChannelPacket, CommandLine, Direction, FRAGMENT_BYTES, InfoLimit, InfoString, InfoWriter,
    Packet, Route,
};

fn decode_hex(hex: &str) -> Vec<u8> {
    if hex == "~" {
        return Vec::new();
    }
    assert_eq!(hex.len() % 2, 0, "Fixture uses two digits per byte");
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("Fixture is ASCII");
            u8::from_str_radix(text, 16).expect("Fixture is hexadecimal")
        })
        .collect()
}

fn rows(kind: &str) -> impl Iterator<Item = Vec<&'static str>> {
    include_str!("fixtures/wire_vectors.txt")
        .lines()
        .filter(move |line| line.starts_with(kind))
        .map(|line| line.split_ascii_whitespace().collect())
}

#[test]
fn channel_frames_match_original_netchan_transmit_in_both_directions() {
    let lengths = [0, 1, 1299, 1300, 1301, 2600];
    let mut indices = [0; 12];
    let mut count = 0;
    for row in rows("packet ") {
        let case: usize = row[1].parse().expect("Case index");
        let bytes = decode_hex(row[2]);
        let length = lengths[case % 6];
        let (direction, route) = if case < 6 {
            (Direction::ClientToServer, Route::ToServer { qport: 0xbeef })
        } else {
            (Direction::ServerToClient, Route::ToClient)
        };
        let fragmented = length >= FRAGMENT_BYTES;
        let start = indices[case] * FRAGMENT_BYTES;
        let end = (start + FRAGMENT_BYTES).min(length);
        let body: Vec<u8> = (start..end)
            .map(|index| u8::try_from((index * 7 + 3) % 256).expect("Byte range"))
            .collect();
        let offset = fragmented.then(|| u32::try_from(start).expect("Bounded fixture"));
        let expected = ChannelPacket::try_new(0x1234_5678, route, offset, &body)
            .expect("Valid reference envelope");
        assert_eq!(
            Packet::parse(&bytes, direction),
            Ok(Packet::Channel(expected))
        );
        let mut output = vec![0xa5; bytes.len()];
        assert_eq!(
            Packet::Channel(expected).encode(&mut output),
            Ok(bytes.len())
        );
        assert_eq!(output, bytes, "Reference transmit mismatch, case {case}");
        indices[case] += 1;
        count += 1;
    }
    assert_eq!(indices, [1, 1, 1, 2, 2, 3, 1, 1, 1, 2, 2, 3]);
    assert_eq!(count, 20, "Includes zero-length terminal fragments");
}

#[test]
fn command_tokens_match_original_tokenizer() {
    let mut count = 0;
    for row in rows("tokens ") {
        let input = decode_hex(row[1]);
        let expected_count: usize = row[2].parse().expect("Token count");
        let expected: Vec<Vec<u8>> = if expected_count == 0 {
            Vec::new()
        } else {
            row[3].split(',').map(decode_hex).collect()
        };
        assert_eq!(expected.len(), expected_count);
        let command = CommandLine::parse(&input).expect("Bounded command");
        let actual: Vec<&[u8]> = command.tokens().collect();
        let expected: Vec<&[u8]> = expected.iter().map(Vec::as_slice).collect();
        assert_eq!(
            actual, expected,
            "Token mismatch for reference case {count}"
        );
        count += 1;
    }
    assert_eq!(count, 12);
}

#[test]
fn info_replacement_order_and_lookup_match_original_functions() {
    let operations: [(&[u8], &[u8]); 8] = [
        (b"hostname", b"Rust"),
        (b"protocol", b"6"),
        (b"hostname", b"Updated"),
        (b"Key", b"One"),
        (b"key", b"Two"),
        (b"Key", b""),
        (b"mapname", b"mp_crash"),
        (b"key", b"Final"),
    ];
    let mut storage = [0xa5; 1024];
    let mut writer = InfoWriter::new(&mut storage, InfoLimit::Normal);
    let mut count = 0;
    for row in rows("info ") {
        let index: usize = row[1].parse().expect("Operation index");
        let (key, value) = operations[index];
        writer.set(key, value).expect("Bounded valid field");
        let expected = decode_hex(row[2]);
        assert_eq!(writer.as_bytes(), expected);
        let view = InfoString::parse(writer.as_bytes(), InfoLimit::Normal)
            .expect("Writer must generate valid strings");
        let expected_lookup = decode_hex(row[3]);
        assert_eq!(view.get(b"KEY").unwrap_or(&[]), expected_lookup);
        count += 1;
    }
    assert_eq!(count, operations.len());
}
