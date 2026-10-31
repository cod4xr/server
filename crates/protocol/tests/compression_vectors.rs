// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original compressor/decompressor fixtures, including final padding semantics.
#![allow(
    clippy::expect_used,
    reason = "Source-generated fixtures must be well formed."
)]

use cod4x_protocol::compression;

fn decode_hex(hex: &str) -> Vec<u8> {
    if hex == "~" {
        return Vec::new();
    }
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("ASCII fixture");
            u8::from_str_radix(text, 16).expect("Hex byte")
        })
        .collect()
}

#[test]
fn compression_and_safe_decompression_match_original_c_including_padding_symbols() {
    let mut count = 0;
    for line in include_str!("fixtures/huffman_vectors.txt").lines() {
        if !line.starts_with("sample ") {
            continue;
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        let plain = decode_hex(fields[3]);
        let encoded = decode_hex(fields[4]);
        let decoded = decode_hex(fields[5]);
        let mut output = vec![0xa5; encoded.len() + 1];
        assert_eq!(
            compression::compress(&plain, &mut output),
            Ok(encoded.len())
        );
        assert_eq!(&output[..encoded.len()], encoded);
        assert_eq!(output[encoded.len()], 0xa5);
        let mut output = vec![0xa5; decoded.len() + 1];
        assert_eq!(
            compression::decompress(&encoded, &mut output),
            Ok(decoded.len())
        );
        assert_eq!(&output[..decoded.len()], decoded);
        assert_eq!(output[decoded.len()], 0xa5);
        assert!(decoded.starts_with(&plain));
        count += 1;
    }
    assert_eq!(count, 32);
}

#[test]
fn every_byte_code_matches_the_pinned_tree() {
    let mut count = 0;
    for line in include_str!("fixtures/huffman_vectors.txt").lines() {
        if !line.starts_with("code ") {
            continue;
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        let symbol: u16 = fields[1].parse().expect("Symbol index");
        let Ok(byte) = u8::try_from(symbol) else {
            continue;
        };
        let expected = decode_hex(fields[3]);
        let mut output = [0xa5; 2];
        let length = compression::compress(&[byte], &mut output).expect("At most 11 bits");
        assert_eq!(&output[..length], expected);
        count += 1;
    }
    assert_eq!(count, 256);
}
