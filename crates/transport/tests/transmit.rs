// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reference packetization, cancellation, and packet-buffer retry contracts.
#![allow(
    clippy::expect_used,
    reason = "Tests use bounded source-generated fixtures."
)]

use cod4x_protocol::{MAX_MESSAGE_BYTES, Packet, Route};
use cod4x_transport::{ChannelError, SendChannel};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("Hex fixture");
            u8::from_str_radix(text, 16).expect("Hexadecimal byte")
        })
        .collect()
}

#[test]
fn packetization_matches_original_c_transmission_boundaries_and_bytes() {
    let mut expected: Vec<Vec<Vec<u8>>> = vec![Vec::new(); 12];
    let rows = include_str!("../../protocol/tests/fixtures/wire_vectors.txt").lines();
    for row in rows {
        if !row.starts_with("packet ") {
            continue;
        }
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let case: usize = fields[1].parse().expect("Reference case index");
        let mut bytes = decode_hex(fields[2]);
        // The original C fixture uses a deliberately visible sequence. The
        // sender starts at one; change only that independently known field.
        let sequence = 1_u32 | (u32::from(bytes[3] & 0x80) << 24);
        bytes[..4].copy_from_slice(&sequence.to_le_bytes());
        expected[case].push(bytes);
    }
    let lengths = [0, 1, 1299, 1300, 1301, 2600];
    for (case, packets) in expected.iter().enumerate() {
        let route = if case < 6 {
            Route::ToServer { qport: 0xbeef }
        } else {
            Route::ToClient
        };
        let body: Vec<u8> = (0..lengths[case % 6])
            .map(|index| u8::try_from((index * 7 + 3) % 256).expect("Byte range"))
            .collect();
        let mut sender = SendChannel::new(route);
        {
            let mut transmission = sender.begin(&body).expect("Bounded message");
            let mut output = [0xa5; 1400];
            for expected in packets {
                let length = transmission
                    .next_packet(&mut output)
                    .expect("Output fits")
                    .expect("Reference packet exists");
                assert_eq!(
                    &output[..length],
                    expected,
                    "Reference mismatch for case {case}"
                );
            }
            assert_eq!(transmission.next_packet(&mut output), Ok(None));
            assert_eq!(transmission.next_packet(&mut output), Ok(None));
        }
        assert_eq!(sender.next_sequence(), 2);
    }
}

#[test]
fn failed_packet_encoding_can_retry_and_cancelled_sequences_are_never_reused() {
    let mut sender = SendChannel::new(Route::ToClient);
    let body = [7; 1300];
    {
        let mut transmission = sender.begin(&body).expect("Bounded message");
        let mut small = [0xa5; 4];
        assert!(transmission.next_packet(&mut small).is_err());
        assert_eq!(small, [0xa5; 4]);
        let mut output = [0; 1400];
        let length = transmission
            .next_packet(&mut output)
            .expect("Retry fits")
            .expect("First fragment");
        assert_eq!(length, 1310);
        // Cancellation after sending the prefix must not reuse sequence one.
    }
    assert_eq!(sender.next_sequence(), 2);
    let mut transmission = sender.begin(b"next").expect("Next message");
    let mut output = [0; 32];
    let length = transmission
        .next_packet(&mut output)
        .expect("Fits")
        .expect("Packet");
    assert_eq!(&output[..4], &2_u32.to_le_bytes());
    assert!(
        matches!(Packet::parse(&output[..length], cod4x_protocol::Direction::ServerToClient),
        Ok(Packet::Channel(packet)) if packet.sequence() == 2 && packet.body() == b"next")
    );
}

#[test]
fn oversize_begin_does_not_reserve_a_sequence() {
    let mut sender = SendChannel::new(Route::ToClient);
    assert!(matches!(
        sender.begin(&vec![0; MAX_MESSAGE_BYTES + 1]),
        Err(ChannelError::MessageTooLarge)
    ));
    assert_eq!(sender.next_sequence(), 1);
}
