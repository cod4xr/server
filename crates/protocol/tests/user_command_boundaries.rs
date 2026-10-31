// SPDX-License-Identifier: AGPL-3.0-or-later
//! Invalid baseline and legacy timestamp/angle boundary behavior.
#![allow(
    clippy::expect_used,
    reason = "Tests exercise exact public decoder boundaries."
)]

use cod4x_protocol::MessageReader;
use cod4x_protocol::user_command::{self, LegacyUserCommand, UserCommandError};

#[test]
fn invalid_baselines_are_rejected_without_consuming_input_or_logging_contents() {
    let valid = LegacyUserCommand::default();
    for baseline in [
        LegacyUserCommand {
            buttons: 0x0020_0000,
            ..valid
        },
        LegacyUserCommand {
            weapon: 128,
            ..valid
        },
        LegacyUserCommand {
            offhand: 255,
            ..valid
        },
        LegacyUserCommand {
            melee_charge_yaw: f32::NAN,
            ..valid
        },
        LegacyUserCommand {
            melee_charge_yaw: f32::INFINITY,
            ..valid
        },
        LegacyUserCommand {
            melee_charge_yaw: f32::NEG_INFINITY,
            ..valid
        },
        LegacyUserCommand {
            melee_charge_yaw: f32::MAX,
            ..valid
        },
        LegacyUserCommand {
            melee_charge_yaw: -f32::MAX,
            ..valid
        },
    ] {
        let mut reader = MessageReader::try_new(&[0x01, 0xff]).expect("Input");
        assert_eq!(
            user_command::read_delta(&mut reader, 0, &baseline),
            Err(UserCommandError::InvalidBaseline)
        );
        assert_eq!(reader.remaining_bytes(), 2);
        assert_eq!(reader.read_bits(1).expect("Original first bit"), 1);
        assert_eq!(reader.read_u8().expect("Original second byte"), 255);
    }
}

#[test]
fn timestamp_delta_wraps_explicitly_and_unchanged_fields_keep_float_bits() {
    let baseline = LegacyUserCommand {
        server_time: i32::MAX,
        melee_charge_yaw: -0.0,
        selected_location: [255, 19],
        angles: [65535, 10, 32768],
        ..LegacyUserCommand::default()
    };
    // First flag selects byte delta; keyed changed flag is zero in the same bit byte.
    let mut reader = MessageReader::try_new(&[1, 1]).expect("Input");
    let decoded = user_command::read_delta(&mut reader, 0, &baseline).expect("Time-only delta");
    assert_eq!(decoded.server_time, i32::MIN);
    assert_eq!(decoded.melee_charge_yaw.to_bits(), (-0.0_f32).to_bits());
    assert_eq!(decoded.angles, baseline.angles);
    assert_eq!(decoded.selected_location, baseline.selected_location);
    assert_eq!(reader.remaining_bytes(), 0);
}
