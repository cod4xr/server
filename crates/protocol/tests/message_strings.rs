// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy string bytes, terminators, split boundaries, and bounded failures.
#![allow(
    clippy::expect_used,
    reason = "Tests construct validated byte-string fixtures."
)]

use cod4x_protocol::{MessageError, MessageReader, MessageStringLimit, MessageWriter};

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
fn bytes_and_percent_normalization_match_original_functions_at_every_split() {
    let mut count = 0;
    for line in include_str!("fixtures/wire_vectors.txt").lines() {
        if !line.starts_with("string ") {
            continue;
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        let limit = if fields[1] == "0" {
            MessageStringLimit::Normal
        } else {
            MessageStringLimit::Big
        };
        let encoded = decode_hex(fields[2]);
        let original = decode_hex(fields[3]);
        let decoded = decode_hex(fields[4]);
        let mut storage = [0xa5; 128];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Storage");
        writer
            .write_legacy_string(&original, limit)
            .expect("Valid field");
        assert_eq!(writer.as_bytes(), encoded);
        for split in 0..=encoded.len() {
            let mut reader =
                MessageReader::try_new_split(&encoded[..split], &encoded[split..]).expect("Packet");
            let mut output = vec![0xa5; decoded.len() + 1];
            assert_eq!(
                reader.read_legacy_string(&mut output, limit),
                Ok(decoded.len())
            );
            assert_eq!(&output[..decoded.len()], decoded);
            assert_eq!(output[decoded.len()], 0xa5);
            assert_eq!(reader.remaining_bytes(), 0);
        }
        count += 1;
    }
    assert_eq!(count, 12);
}

#[test]
fn exact_profile_limits_consume_the_terminator_before_the_following_field() {
    for limit in [MessageStringLimit::Normal, MessageStringLimit::Big] {
        let bytes = vec![b'x'; limit.maximum_bytes()];
        let mut storage = vec![0xa5; bytes.len() + 2];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Storage");
        writer
            .write_legacy_string(&bytes, limit)
            .expect("Exact profile limit");
        writer.write_u8(7).expect("Following field");
        let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Packet");
        let mut output = vec![0; bytes.len()];
        assert_eq!(
            reader.read_legacy_string(&mut output, limit),
            Ok(bytes.len())
        );
        assert_eq!(output, bytes);
        assert_eq!(reader.read_u8(), Ok(7));
    }
}

#[test]
fn strings_preserve_independent_byte_and_bit_cursors() {
    let mut storage = [0; 32];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Storage");
    writer.write_bits(5, 3).expect("Initial bits");
    writer
        .write_legacy_string(b"a%b", MessageStringLimit::Normal)
        .expect("String");
    writer.write_bits(27, 5).expect("Cached bits");
    assert_eq!(writer.as_bytes(), &[0xdd, b'a', b'%', b'b', 0]);
    let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Packet");
    assert_eq!(reader.read_bits(3), Ok(5));
    let mut output = [0; 3];
    assert_eq!(
        reader.read_legacy_string(&mut output, MessageStringLimit::Normal),
        Ok(3)
    );
    assert_eq!(&output, b"a.b");
    assert_eq!(reader.read_bits(5), Ok(27));
}

#[test]
fn read_failures_preserve_output_and_reader_state() {
    for input in [b"abc".as_slice(), b"abc\0"] {
        let mut reader = MessageReader::try_new(input).expect("Packet");
        let mut output = [0xa5; 2];
        assert!(
            reader
                .read_legacy_string(&mut output, MessageStringLimit::Normal)
                .is_err()
        );
        assert_eq!(output, [0xa5; 2]);
        assert_eq!(reader.read_u8(), Ok(b'a'));
    }
    let mut input = vec![b'x'; MessageStringLimit::Normal.maximum_bytes() + 1];
    input.push(0);
    let mut reader = MessageReader::try_new(&input).expect("Packet");
    let mut output = [0xa5; 2048];
    assert!(matches!(
        reader.read_legacy_string(&mut output, MessageStringLimit::Normal),
        Err(MessageError::StringTooLong { .. })
    ));
    assert_eq!(output, [0xa5; 2048]);
    assert_eq!(reader.read_u8(), Ok(b'x'));
}

#[test]
fn write_failures_preserve_previous_bytes_and_reject_embedded_nul() {
    let mut storage = [0xa5; 4];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Storage");
    writer.write_u8(7).expect("Prefix");
    assert!(
        writer
            .write_legacy_string(b"abc", MessageStringLimit::Normal)
            .is_err()
    );
    assert_eq!(writer.as_bytes(), &[7]);
    assert_eq!(
        writer.write_legacy_string(b"a\0b", MessageStringLimit::Normal),
        Err(MessageError::EmbeddedTerminator)
    );
    assert_eq!(writer.as_bytes(), &[7]);
}
