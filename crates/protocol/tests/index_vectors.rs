// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-generated legacy index streams, independent of engine allocation limits.
#![allow(
    clippy::expect_used,
    reason = "Original C fixtures have validated framing."
)]

use cod4x_protocol::{
    LegacyIndexDecoder, LegacyIndexEncoder, LegacyIndexKind, MessageReader, MessageWriter,
};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16).expect("Hex byte")
        })
        .collect()
}

#[test]
fn ordered_entity_and_client_indices_match_original_writers_and_readers() {
    let mut count = 0;
    for line in include_str!("fixtures/wire_vectors.txt").lines() {
        if !line.starts_with("index ") {
            continue;
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        let kind = match fields[1] {
            "6" => LegacyIndexKind::Client,
            "10" => LegacyIndexKind::Entity,
            _ => unreachable!("Fixture uses the audited profile"),
        };
        let expected = decode_hex(fields[2]);
        let indices: Vec<u16> = fields[3]
            .split(',')
            .map(|index| index.parse().expect("Representable index"))
            .collect();
        let mut storage = [0xa5; 512];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Bounded storage");
        let mut encoder = LegacyIndexEncoder::new(kind);
        for &index in &indices {
            encoder
                .write(index, &mut writer)
                .expect("Increasing valid index");
        }
        assert_eq!(writer.as_bytes(), expected);
        for split in 0..=expected.len() {
            let mut reader = MessageReader::try_new_split(&expected[..split], &expected[split..])
                .expect("Bounded input");
            let mut decoder = LegacyIndexDecoder::new(kind);
            for &index in &indices {
                assert_eq!(decoder.read(&mut reader), Ok(index));
            }
        }
        count += 1;
    }
    assert_eq!(count, 64);
}
