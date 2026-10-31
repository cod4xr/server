// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hostile input, atomic failure, exact limits, and known representation tests.
#![allow(
    clippy::expect_used,
    reason = "Tests assert fixture setup and successful recovery."
)]

use cod4x_protocol::{MAX_MESSAGE_BYTES, MessageError, MessageReader, MessageWriter};

#[test]
fn mixed_cursors_match_a_hand_calculated_packet() {
    let mut storage = [0xa5; 16];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Small buffer");
    writer.write_bits(5, 3).expect("Three bits");
    writer.write_u8(0xaa).expect("Byte");
    writer.write_bits(27, 5).expect("Five cached bits");
    writer.write_i16(-2).expect("Signed integer");
    writer.write_bits(0x123, 9).expect("Cross-byte bits");
    assert_eq!(writer.as_bytes(), &[0xdd, 0xaa, 0xfe, 0xff, 0x23, 0x01]);
}

#[test]
fn cached_bits_remain_readable_after_all_new_bytes_are_consumed() {
    let mut reader = MessageReader::try_new(&[0xdd, 0xaa]).expect("Small packet");
    assert_eq!(reader.read_bits(3), Ok(5));
    assert_eq!(reader.read_u8(), Ok(0xaa));
    assert_eq!(reader.remaining_bytes(), 0);
    assert_eq!(reader.read_bits(5), Ok(27));
    assert!(matches!(
        reader.read_bits(1),
        Err(MessageError::Truncated { .. })
    ));
}

#[test]
fn full_buffer_can_accept_cached_bits_without_allocating() {
    let mut storage = [0xa5; 1];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Small buffer");
    writer.write_bits(5, 3).expect("Three bits");
    assert_eq!(writer.remaining_capacity(), 0);
    assert_eq!(writer.write_bits(27, 5), Ok(()));
    assert_eq!(writer.as_bytes(), &[0xdd]);
}

#[test]
fn failed_bit_write_preserves_partial_byte_and_cursor() {
    let mut storage = [0xff; 1];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Small buffer");
    writer.write_bits(1, 1).expect("One bit");
    assert_eq!(
        writer.write_bits(0xff, 8),
        Err(MessageError::Capacity {
            needed: 1,
            remaining: 0
        })
    );
    assert_eq!(writer.as_bytes(), &[1]);
    writer
        .write_bits(0x55, 7)
        .expect("Failed operation must not advance the cursor");
    assert_eq!(writer.as_bytes(), &[0xab]);
}

#[test]
fn failed_bit_read_preserves_partial_byte_and_cursor() {
    let mut reader = MessageReader::try_new(&[0xab]).expect("Small packet");
    assert_eq!(reader.read_bits(1), Ok(1));
    assert_eq!(
        reader.read_bits(8),
        Err(MessageError::Truncated {
            needed: 1,
            remaining: 0
        })
    );
    assert_eq!(reader.read_bits(7), Ok(0x55));
}

#[test]
fn failed_byte_operations_leave_output_and_cursors_unchanged() {
    let mut reader = MessageReader::try_new_split(&[1], &[2]).expect("Small packet");
    let mut output = [0xa5; 3];
    assert!(reader.read_bytes(&mut output).is_err());
    assert_eq!(output, [0xa5; 3]);
    assert!(reader.read_i32().is_err());
    assert_eq!(reader.read_i16(), Ok(513));

    let mut storage = [0xa5; 2];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Small buffer");
    assert!(writer.write_bytes(&[1, 2, 3]).is_err());
    assert!(writer.write_i32(-1).is_err());
    assert!(writer.is_empty());
    writer.write_i16(513).expect("Failed write preserves space");
    assert_eq!(writer.as_bytes(), &[1, 2]);
}

#[test]
fn zero_width_and_empty_slices_work_with_zero_capacity() {
    let mut reader = MessageReader::try_new(&[]).expect("Empty message is valid");
    assert_eq!(reader.read_bits(0), Ok(0));
    assert_eq!(reader.read_bytes(&mut []), Ok(()));
    assert!(reader.read_u8().is_err());
    let mut writer = MessageWriter::try_new(&mut []).expect("Empty buffer is valid");
    assert_eq!(writer.write_bits(u32::MAX, 0), Ok(()));
    assert_eq!(writer.write_bytes(&[]), Ok(()));
    assert!(writer.write_u8(0).is_err());
    assert_eq!(writer.len(), 0);
}

#[test]
fn every_invalid_width_is_rejected_without_mutation() {
    for width in 33..=u8::MAX {
        let mut reader = MessageReader::try_new(&[0xab]).expect("Small packet");
        assert_eq!(
            reader.read_bits(width),
            Err(MessageError::InvalidBitWidth { width })
        );
        assert_eq!(reader.read_bits(8), Ok(0xab));
        let mut storage = [0xa5; 4];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Small buffer");
        assert_eq!(
            writer.write_bits(1, width),
            Err(MessageError::InvalidBitWidth { width })
        );
        assert!(writer.is_empty());
        assert_eq!(writer.write_bits(0xab, 8), Ok(()));
        assert_eq!(writer.as_bytes(), &[0xab]);
    }
}

#[test]
fn all_widths_and_alignments_obey_capacity_and_truncation_limits() {
    for prefix in 0_u8..8 {
        for width in 0_u8..=32 {
            let required = usize::from(prefix + width).div_ceil(8);
            for capacity in 0..=6 {
                let mut storage = [0xa5; 6];
                let mut writer =
                    MessageWriter::try_new(&mut storage[..capacity]).expect("Small buffer");
                if writer.write_bits(0, prefix).is_err() {
                    continue;
                }
                let previous = writer.as_bytes().to_vec();
                let result = writer.write_bits(u32::MAX, width);
                assert_eq!(result.is_ok(), required <= capacity);
                if result.is_err() {
                    assert_eq!(writer.as_bytes(), previous);
                }
                let input = [0xff; 6];
                let mut reader = MessageReader::try_new(&input[..capacity]).expect("Small packet");
                reader
                    .read_bits(prefix)
                    .expect("Same prefix capacity as writer");
                assert_eq!(reader.read_bits(width).is_ok(), required <= capacity);
            }
        }
    }
}

#[test]
fn exact_message_limit_is_enforced_for_contiguous_and_split_storage() {
    let mut storage = vec![0; MAX_MESSAGE_BYTES + 1];
    assert!(matches!(
        MessageWriter::try_new(&mut storage),
        Err(MessageError::MessageTooLarge { .. })
    ));
    assert!(matches!(
        MessageReader::try_new(&storage),
        Err(MessageError::MessageTooLarge { .. })
    ));
    assert!(matches!(
        MessageReader::try_new_split(&storage[..MAX_MESSAGE_BYTES], &[1]),
        Err(MessageError::MessageTooLarge { .. })
    ));
    let mut writer =
        MessageWriter::try_new(&mut storage[..MAX_MESSAGE_BYTES]).expect("Exact limit");
    writer
        .write_bytes(&vec![0x42; MAX_MESSAGE_BYTES])
        .expect("Exact fit");
    assert!(writer.write_u8(0).is_err());
    let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Exact limit");
    let mut output = vec![0; MAX_MESSAGE_BYTES];
    reader.read_bytes(&mut output).expect("Exact fit");
    assert!(output.iter().all(|&byte| byte == 0x42));
}

#[test]
fn floating_point_wire_bits_preserve_special_values_at_every_split() {
    let patterns = [
        0_u32,
        0x8000_0000,
        1,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_1234,
        0x7f80_1234,
    ];
    for pattern in patterns {
        let bytes = pattern.to_le_bytes();
        let mut storage = [0; 4];
        let mut writer = MessageWriter::try_new(&mut storage).expect("Float buffer");
        writer
            .write_f32(f32::from_bits(pattern))
            .expect("Exact fit");
        assert_eq!(writer.as_bytes(), bytes);
        for boundary in 0..=4 {
            let mut reader = MessageReader::try_new_split(&bytes[..boundary], &bytes[boundary..])
                .expect("Float packet");
            assert_eq!(
                reader.read_f32().expect("Complete float").to_bits(),
                pattern
            );
        }
    }
}

#[test]
fn reuse_resets_both_cursors_and_clears_stale_bits() {
    let mut storage = [0xff; 8];
    let mut writer = MessageWriter::try_new(&mut storage).expect("Small buffer");
    writer.write_bits(u32::MAX, 31).expect("First message");
    writer.clear();
    writer.write_u8(0xaa).expect("Second message header");
    writer
        .write_bits(0, 1)
        .expect("New bit byte must start at zero");
    assert_eq!(writer.as_bytes(), &[0xaa, 0]);
    let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Second message");
    assert_eq!(reader.read_u8(), Ok(0xaa));
    assert_eq!(reader.read_bits(1), Ok(0));
    reader.rewind();
    assert_eq!(reader.read_u8(), Ok(0xaa));
    assert_eq!(reader.read_bits(1), Ok(0));
}
