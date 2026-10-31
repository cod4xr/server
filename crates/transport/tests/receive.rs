// SPDX-License-Identifier: AGPL-3.0-or-later
//! Loss/reordering/duplicate handling and bounded assembly invariants.
#![allow(
    clippy::expect_used,
    reason = "Tests construct validated channel envelopes."
)]

use cod4x_protocol::{ChannelPacket, Direction, MAX_MESSAGE_BYTES, Packet, Route};
use cod4x_transport::{ReceiveChannel, ReceiveDrop, ReceiveOutcome, SendChannel};

#[test]
fn both_directions_reassemble_every_boundary_through_the_maximum_message_size() {
    for route in [Route::ToServer { qport: 0xbeef }, Route::ToClient] {
        let direction = match route {
            Route::ToServer { .. } => Direction::ClientToServer,
            Route::ToClient => Direction::ServerToClient,
        };
        let mut sender = SendChannel::new(route);
        let mut storage = vec![0xa5; MAX_MESSAGE_BYTES];
        let mut receiver = ReceiveChannel::try_new(&mut storage, route).expect("Bounded storage");
        for length in [0, 1, 1299, 1300, 1301, 2600, 65_536, MAX_MESSAGE_BYTES] {
            let body: Vec<u8> = (0..length)
                .map(|index| u8::try_from(index % 251).expect("Byte range"))
                .collect();
            let mut transmission = sender.begin(&body).expect("Bounded message");
            let mut output = [0; 1400];
            let mut delivered = 0;
            while let Some(length) = transmission.next_packet(&mut output).expect("Packet fits") {
                let packet = Packet::parse(&output[..length], direction).expect("Valid frame");
                let Packet::Channel(packet) = packet else {
                    unreachable!("Sender emits channel packets");
                };
                match receiver.accept(&packet) {
                    ReceiveOutcome::Message(message) => {
                        assert_eq!(message.body(), body);
                        assert_eq!(message.skipped_sequences(), 0);
                        delivered += 1;
                    }
                    ReceiveOutcome::Pending => {}
                    other => unreachable!("Sequential valid packets must be accepted: {other:?}"),
                }
            }
            assert_eq!(
                delivered,
                1,
                "One logical message for length {}",
                body.len()
            );
            assert_eq!(receiver.partial_bytes(), 0);
        }
        assert_eq!(receiver.incoming_sequence(), 8);
    }
}

#[test]
fn missing_and_duplicate_fragments_preserve_the_prefix_for_retransmission() {
    let mut storage = [0xa5; 4096];
    let mut receiver = ReceiveChannel::try_new(&mut storage, Route::ToClient).expect("Storage");
    let first = ChannelPacket::try_new(1, Route::ToClient, Some(0), &[1; 1300]).expect("First");
    let middle =
        ChannelPacket::try_new(1, Route::ToClient, Some(1300), &[2; 1300]).expect("Middle");
    let last = ChannelPacket::try_new(1, Route::ToClient, Some(2600), &[3; 10]).expect("Last");
    assert_eq!(receiver.accept(&first), ReceiveOutcome::Pending);
    assert_eq!(
        receiver.accept(&last),
        ReceiveOutcome::Dropped(ReceiveDrop::OutOfOrderFragment)
    );
    assert_eq!(
        receiver.accept(&first),
        ReceiveOutcome::Dropped(ReceiveDrop::OutOfOrderFragment)
    );
    assert_eq!(receiver.incoming_sequence(), 0);
    assert_eq!(receiver.partial_bytes(), 1300);
    assert_eq!(receiver.accept(&middle), ReceiveOutcome::Pending);
    let ReceiveOutcome::Message(message) = receiver.accept(&last) else {
        unreachable!("Retransmitted final fragment must complete the message");
    };
    assert_eq!(&message.body()[..1300], &[1; 1300]);
    assert_eq!(&message.body()[1300..2600], &[2; 1300]);
    assert_eq!(&message.body()[2600..], &[3; 10]);
    assert_eq!(
        receiver.accept(&last),
        ReceiveOutcome::Dropped(ReceiveDrop::StaleSequence)
    );
}

#[test]
fn capacity_wrong_route_and_invalid_new_offsets_do_not_destroy_partial_data() {
    let mut storage = [0xa5; 1400];
    let mut receiver = ReceiveChannel::try_new(&mut storage, Route::ToClient).expect("Storage");
    let first = ChannelPacket::try_new(1, Route::ToClient, Some(0), &[1; 1300]).expect("First");
    assert_eq!(receiver.accept(&first), ReceiveOutcome::Pending);
    let large = ChannelPacket::try_new(1, Route::ToClient, Some(1300), &[2; 1300]).expect("Large");
    assert_eq!(
        receiver.accept(&large),
        ReceiveOutcome::Dropped(ReceiveDrop::Capacity)
    );
    let newer = ChannelPacket::try_new(2, Route::ToClient, Some(100), &[3]).expect("Newer");
    assert_eq!(
        receiver.accept(&newer),
        ReceiveOutcome::Dropped(ReceiveDrop::OutOfOrderFragment)
    );
    let wrong =
        ChannelPacket::try_new(2, Route::ToServer { qport: 1 }, None, &[3]).expect("Wrong route");
    assert_eq!(
        receiver.accept(&wrong),
        ReceiveOutcome::Dropped(ReceiveDrop::WrongRoute)
    );
    let last = ChannelPacket::try_new(1, Route::ToClient, Some(1300), &[4; 100]).expect("Last");
    let ReceiveOutcome::Message(message) = receiver.accept(&last) else {
        unreachable!("Rejected packets must leave the valid prefix intact");
    };
    assert_eq!(&message.body()[..1300], &[1; 1300]);
    assert_eq!(&message.body()[1300..], &[4; 100]);
}

#[test]
fn newer_sequences_replace_partial_messages_and_older_fragments_stay_stale() {
    let mut storage = [0; 4096];
    let mut receiver = ReceiveChannel::try_new(&mut storage, Route::ToClient).expect("Storage");
    let old = ChannelPacket::try_new(2, Route::ToClient, Some(0), &[2; 1300]).expect("Old");
    let new = ChannelPacket::try_new(3, Route::ToClient, Some(0), &[3; 1300]).expect("New");
    assert_eq!(receiver.accept(&old), ReceiveOutcome::Pending);
    assert_eq!(receiver.accept(&new), ReceiveOutcome::Pending);
    assert_eq!(
        receiver.accept(&old),
        ReceiveOutcome::Dropped(ReceiveDrop::StaleSequence)
    );
    let last = ChannelPacket::try_new(3, Route::ToClient, Some(1300), &[]).expect("Terminator");
    let ReceiveOutcome::Message(message) = receiver.accept(&last) else {
        unreachable!("An empty terminal fragment must complete an exact multiple");
    };
    assert_eq!(message.body(), &[3; 1300]);
    assert_eq!(message.sequence(), 3);
    assert_eq!(message.skipped_sequences(), 2);
    receiver.discard_partial();
    assert_eq!(receiver.incoming_sequence(), 3);
}

#[test]
fn an_older_complete_packet_can_arrive_during_newer_reassembly_without_overwriting_it() {
    let mut storage = [0; 4096];
    let mut receiver = ReceiveChannel::try_new(&mut storage, Route::ToClient).expect("Storage");
    let first = ChannelPacket::try_new(3, Route::ToClient, Some(0), &[3; 1300]).expect("Prefix");
    assert_eq!(receiver.accept(&first), ReceiveOutcome::Pending);
    let older = ChannelPacket::try_new(2, Route::ToClient, None, b"older").expect("Older packet");
    let ReceiveOutcome::Message(message) = receiver.accept(&older) else {
        unreachable!("A complete ordered message can be borrowed without touching the prefix");
    };
    assert_eq!(message.body(), b"older");
    assert_eq!(receiver.partial_bytes(), 1300);
    let last = ChannelPacket::try_new(3, Route::ToClient, Some(1300), &[]).expect("Final");
    let ReceiveOutcome::Message(message) = receiver.accept(&last) else {
        unreachable!("Newer reassembly must remain intact");
    };
    assert_eq!(message.body(), &[3; 1300]);
    assert_eq!(message.skipped_sequences(), 0);
}
