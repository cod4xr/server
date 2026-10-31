// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent envelope integrity and ownership checks; asset graphs stay opaque.
#![allow(
    clippy::expect_used,
    reason = "Tests decode known independent fixtures."
)]

mod fastfile_support;

use cod4x_content::fastfile::{FastFileBudget, FastFileStream};

#[test]
fn independent_streams_have_exact_payloads_and_widened_metadata() {
    for name in ["empty", "small", "pattern", "expansion"] {
        let row = fastfile_support::row(name);
        let source = fastfile_support::fixture(name);
        let original = source.clone();
        let stream = FastFileStream::decode(&source, FastFileBudget::default()).expect("Stream");
        assert_eq!(source, original);
        let expected_size: u64 = row[2].parse().expect("Size");
        let expected_crc: u32 = row[3].parse().expect("CRC");
        assert_eq!(stream.header().payload_bytes(), expected_size);
        assert_eq!(stream.payload().len() as u64, expected_size);
        assert_eq!(crc32fast::hash(stream.payload()), expected_crc);
        assert_eq!(stream.header().external_bytes(), 1_223_352);
        assert_eq!(
            stream.header().block_bytes(),
            &[164, 0, 0, 0, 223, 0, 0, 0, 0]
        );
        assert_eq!(stream.header().total_block_bytes(), 387);
        let expected: Vec<u8> = match name {
            "small" => b"synthetic-secret-asset\0\xff\x80".to_vec(),
            "pattern" => (0..150_123)
                .map(|index| u8::try_from(index % 251).expect("Byte"))
                .collect(),
            "expansion" => vec![b'Z'; 200_003],
            _ => Vec::new(),
        };
        assert_eq!(stream.into_payload(), expected);
    }
}

#[test]
fn owned_stream_survives_source_drop_and_debug_excludes_asset_bytes() {
    let stream = FastFileStream::decode(
        &fastfile_support::fixture("small"),
        FastFileBudget::default(),
    )
    .expect("Owned output");
    let debug = format!("{stream:?}");
    assert!(debug.contains("payload_bytes"));
    assert!(!debug.contains("secret"));
    assert!(!debug.contains("115, 121, 110"));
    assert_eq!(stream.payload(), b"synthetic-secret-asset\0\xff\x80");
}
