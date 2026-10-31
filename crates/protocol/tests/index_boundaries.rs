// SPDX-License-Identifier: AGPL-3.0-or-later
//! Narrowing rejection, stream order, and transactional index cursor state.
#![allow(
    clippy::expect_used,
    reason = "Boundary tests construct known valid prefixes."
)]

use cod4x_protocol::{
    LegacyIndexDecoder, LegacyIndexEncoder, LegacyIndexError, LegacyIndexKind, MessageReader,
    MessageWriter,
};

#[test]
fn every_index_in_each_profile_round_trips_in_an_increment_stream() {
    for kind in [LegacyIndexKind::Client, LegacyIndexKind::Entity] {
        let mut storage = [0xa5; 128];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Storage");
        let mut encoder = LegacyIndexEncoder::new(kind);
        for index in 0..=kind.maximum_index() {
            encoder.write(index, &mut writer).expect("Increasing index");
        }
        let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Packet");
        let mut decoder = LegacyIndexDecoder::new(kind);
        for index in 0..=kind.maximum_index() {
            assert_eq!(decoder.read(&mut reader), Ok(index));
        }
        decoder.reset();
        reader.rewind();
        assert_eq!(decoder.read(&mut reader), Ok(0));
        encoder.reset();
        writer.clear();
        assert_eq!(encoder.write(0, &mut writer), Ok(()));
        assert_eq!(writer.as_bytes(), &[1]);
    }
}

#[test]
fn nonrepresentable_and_nonincreasing_indices_do_not_mutate_encoder_or_output() {
    let mut storage = [0xa5; 8];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Storage");
    let mut encoder = LegacyIndexEncoder::new(LegacyIndexKind::Client);
    encoder.write(5, &mut writer).expect("First index");
    let previous = writer.as_bytes().to_vec();
    for (index, error) in [
        (64, LegacyIndexError::OutOfRange),
        (5, LegacyIndexError::NotIncreasing),
        (4, LegacyIndexError::NotIncreasing),
    ] {
        assert_eq!(encoder.write(index, &mut writer), Err(error));
        assert_eq!(writer.as_bytes(), previous);
    }
    encoder
        .write(6, &mut writer)
        .expect("Failed operations preserve prior index");
    let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Packet");
    let mut decoder = LegacyIndexDecoder::new(LegacyIndexKind::Client);
    assert_eq!(decoder.read(&mut reader), Ok(5));
    assert_eq!(decoder.read(&mut reader), Ok(6));
}

#[test]
fn truncated_multi_field_reads_leave_the_original_bit_cursor_unchanged() {
    let mut reader = MessageReader::try_new(&[2]).expect("Only the full-index prefix fits");
    let mut decoder = LegacyIndexDecoder::new(LegacyIndexKind::Entity);
    assert!(decoder.read(&mut reader).is_err());
    assert_eq!(reader.read_bits(8), Ok(2));
    let mut good = MessageReader::try_new(&[1]).expect("Index zero increment");
    assert_eq!(decoder.read(&mut good), Ok(0));
}

#[test]
fn insufficient_write_storage_does_not_advance_the_index_stream() {
    let mut empty = MessageWriter::try_new(&mut []).expect("Zero capacity");
    let mut encoder = LegacyIndexEncoder::new(LegacyIndexKind::Entity);
    assert!(encoder.write(100, &mut empty).is_err());
    let mut storage = [0; 2];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Enough for full index");
    encoder
        .write(0, &mut writer)
        .expect("Stream still begins at -1");
    assert_eq!(writer.as_bytes(), &[1]);
}

#[test]
fn malformed_increment_past_the_namespace_preserves_the_reader_cursor() {
    for kind in [LegacyIndexKind::Client, LegacyIndexKind::Entity] {
        let mut storage = [0; 4];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Storage");
        let mut encoder = LegacyIndexEncoder::new(kind);
        encoder
            .write(kind.maximum_index(), &mut writer)
            .expect("Last representable index");
        writer
            .write_bits(1, 1)
            .expect("Invalid increment beyond maximum");
        let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Packet");
        let mut decoder = LegacyIndexDecoder::new(kind);
        assert_eq!(decoder.read(&mut reader), Ok(kind.maximum_index()));
        assert_eq!(decoder.read(&mut reader), Err(LegacyIndexError::OutOfRange));
        assert_eq!(reader.read_bits(1), Ok(1));
    }
}
