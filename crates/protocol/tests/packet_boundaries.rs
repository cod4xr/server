// SPDX-License-Identifier: AGPL-3.0-or-later
//! Envelope trust boundaries, marker collisions, and atomic output failures.
#![allow(
    clippy::expect_used,
    reason = "Test envelopes have known valid bounds."
)]

use cod4x_protocol::{
    ChannelPacket, Direction, FRAGMENT_BYTES, MAX_MESSAGE_BYTES, Packet, PacketError, Route,
};

#[test]
fn special_markers_are_classified_before_the_fragment_bit() {
    for direction in [Direction::ClientToServer, Direction::ServerToClient] {
        assert_eq!(
            Packet::parse(&[255, 255, 255, 255, 0, 255], direction),
            Ok(Packet::Connectionless(&[0, 255]))
        );
        assert_eq!(
            Packet::parse(&[254, 255, 255, 255, 4], direction),
            Ok(Packet::UpdateProxy(&[4]))
        );
        assert_eq!(
            Packet::parse(&[240, 255, 255, 255, 0xef, 0xbe, 7], direction),
            Ok(Packet::Reliable {
                qport: 0xbeef,
                body: &[7]
            })
        );
    }
}

#[test]
fn marker_encoding_is_exact_and_has_no_implicit_terminator() {
    let fixtures = [
        (
            Packet::Connectionless(&[1, 0, 2]),
            vec![255, 255, 255, 255, 1, 0, 2],
        ),
        (Packet::UpdateProxy(&[1]), vec![254, 255, 255, 255, 1]),
        (
            Packet::Reliable {
                qport: 0xbeef,
                body: &[1],
            },
            vec![240, 255, 255, 255, 0xef, 0xbe, 1],
        ),
    ];
    for (packet, expected) in fixtures {
        let mut output = vec![0xa5; expected.len() + 1];
        assert_eq!(packet.encode(&mut output), Ok(expected.len()));
        assert_eq!(&output[..expected.len()], expected);
        assert_eq!(output[expected.len()], 0xa5);
    }
}

#[test]
fn every_truncated_header_is_rejected() {
    let server = [1, 0, 0, 0];
    let client = [1, 0, 0, 0, 0xef, 0xbe];
    let fragment = [1, 0, 0, 128, 0xef, 0xbe, 0, 0, 0, 0, 0, 0];
    let reliable = [240, 255, 255, 255, 0xef, 0xbe];
    for (bytes, direction) in [
        (server.as_slice(), Direction::ServerToClient),
        (client.as_slice(), Direction::ClientToServer),
        (fragment.as_slice(), Direction::ClientToServer),
        (reliable.as_slice(), Direction::ClientToServer),
    ] {
        for end in 0..bytes.len() {
            assert!(Packet::parse(&bytes[..end], direction).is_err());
        }
        assert!(Packet::parse(bytes, direction).is_ok());
    }
}

#[test]
fn invalid_sequence_values_and_fragment_marker_collisions_are_rejected() {
    assert_eq!(
        ChannelPacket::try_new(0x8000_0000, Route::ToClient, None, &[]),
        Err(PacketError::InvalidSequence)
    );
    for sequence in [0x7fff_fff0, 0x7fff_fffe, 0x7fff_ffff] {
        assert!(ChannelPacket::try_new(sequence, Route::ToClient, None, &[]).is_ok());
        assert_eq!(
            ChannelPacket::try_new(sequence, Route::ToClient, Some(0), &[]),
            Err(PacketError::InvalidSequence)
        );
    }
}

#[test]
fn invalid_fragment_bounds_and_declared_lengths_are_rejected() {
    let mut bytes = vec![1, 0, 0, 128, 255, 255, 255, 255, 0, 0];
    assert_eq!(
        Packet::parse(&bytes, Direction::ServerToClient),
        Err(PacketError::InvalidFragment)
    );
    bytes[4..8].fill(0);
    bytes[8..10].fill(255);
    assert_eq!(
        Packet::parse(&bytes, Direction::ServerToClient),
        Err(PacketError::InvalidFragment)
    );
    bytes[8..10].copy_from_slice(&1_i16.to_le_bytes());
    assert_eq!(
        Packet::parse(&bytes, Direction::ServerToClient),
        Err(PacketError::FragmentLengthMismatch)
    );
    assert!(
        ChannelPacket::try_new(1, Route::ToClient, Some(0), &vec![0; FRAGMENT_BYTES + 1]).is_err()
    );
    let maximum = u32::try_from(MAX_MESSAGE_BYTES).expect("Protocol limit fits u32");
    assert!(ChannelPacket::try_new(1, Route::ToClient, Some(maximum), &[1]).is_err());
    assert!(ChannelPacket::try_new(1, Route::ToClient, Some(maximum), &[]).is_ok());
}

#[test]
fn all_insufficient_output_sizes_preserve_storage() {
    let channel = ChannelPacket::try_new(1, Route::ToServer { qport: 7 }, Some(0), &[3; 10])
        .expect("Valid fragment");
    let packets = [Packet::Connectionless(&[3; 10]), Packet::Channel(channel)];
    for packet in packets {
        let mut full = [0; 32];
        let length = packet.encode(&mut full).expect("Sufficient storage");
        for capacity in 0..length {
            let mut output = vec![0xa5; capacity];
            assert!(packet.encode(&mut output).is_err());
            assert!(output.iter().all(|&byte| byte == 0xa5));
        }
    }
}
