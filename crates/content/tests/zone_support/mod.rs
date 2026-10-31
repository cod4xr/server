// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests decode checked-in independent fixtures."
)]
#![allow(
    dead_code,
    reason = "Integration tests use different fixture constructors."
)]

use cod4x_content::fastfile::{FastFileBudget, FastFileStream};
use flate2::{Compress, Compression, FlushCompress, Status};

pub fn fixture(name: &str) -> FastFileStream {
    from_dataset(name, include_str!("../fixtures/zones.txt"))
}

pub fn raw_fixture(name: &str) -> FastFileStream {
    from_dataset(name, include_str!("../fixtures/rawfiles.txt"))
}

fn from_dataset(name: &str, dataset: &str) -> FastFileStream {
    let (_, hex) = dataset
        .lines()
        .filter_map(|line| line.split_once(' '))
        .find(|(label, _)| *label == name)
        .expect("Fixture");
    let bytes: Vec<u8> = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16).expect("Byte")
        })
        .collect();
    FastFileStream::decode(&bytes, FastFileBudget::default()).expect("Envelope integrity")
}

pub fn from_payload(payload: &[u8], block_bytes: u32) -> FastFileStream {
    from_payload_with_temp(payload, block_bytes, 0)
}

pub fn from_payload_with_temp(payload: &[u8], block_bytes: u32, temp_bytes: u32) -> FastFileStream {
    let mut plain = vec![0; 44];
    plain[..4].copy_from_slice(
        &u32::try_from(payload.len())
            .expect("Synthetic size")
            .to_le_bytes(),
    );
    plain[24..28].copy_from_slice(&block_bytes.to_le_bytes());
    plain[8..12].copy_from_slice(&temp_bytes.to_le_bytes());
    plain.extend_from_slice(payload);
    let mut compressor = Compress::new(Compression::fast(), true);
    let mut encoded = Vec::with_capacity(plain.len() + 1024);
    assert_eq!(
        compressor
            .compress_vec(&plain, &mut encoded, FlushCompress::Finish)
            .expect("Synthetic compressor"),
        Status::StreamEnd
    );
    let mut envelope = b"IWffu100\x05\0\0\0".to_vec();
    envelope.extend_from_slice(&encoded);
    FastFileStream::decode(&envelope, FastFileBudget::default()).expect("Envelope")
}
