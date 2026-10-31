// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::user_command::keyed_fields;
use crate::user_command::melee_angle;
use crate::user_command::{LegacyUserCommand, UserCommandError};
use crate::{MessageError, MessageReader};

/// Decodes a complete keyed legacy client command against the previous command.
///
/// Preserves the two short/full branches, conditional fields, movement threshold,
/// and branch-specific timestamp rekeying from the pinned server. On any failure,
/// the reader and caller's baseline remain unchanged. No I/O or allocation occurs.
///
/// # Errors
/// Rejects truncated data and invalid caller baselines: buttons above 21 bits,
/// weapon/offhand above 127, or yaw outside defined legacy float-to-int conversion.
pub fn read_delta(
    reader: &mut MessageReader<'_>,
    key: u32,
    baseline: &LegacyUserCommand,
) -> Result<LegacyUserCommand, UserCommandError> {
    if baseline.buttons >= 0x0020_0000 || baseline.weapon >= 128 || baseline.offhand >= 128 {
        return Err(UserCommandError::InvalidBaseline);
    }
    let old_melee_angle = melee_angle::encode(baseline.melee_charge_yaw)?;
    let mut input = reader.clone();
    let mut command = *baseline;
    command.server_time = if input.read_bits(1)? != 0 {
        baseline
            .server_time
            .wrapping_add(i32::from(input.read_u8()?))
    } else {
        input.read_i32()?
    };
    if keyed_fields::bit(&mut input, key)? != 0 {
        command.buttons &= !1;
        if keyed_fields::bit(&mut input, key)? != 0 {
            command.buttons |= keyed_fields::bit(&mut input, key)?;
            read_angles_and_movement(&mut input, key, baseline, &mut command)?;
            let key = key ^ u32::from_le_bytes(command.server_time.to_le_bytes());
            read_full(&mut input, key, baseline, old_melee_angle, &mut command)?;
        } else {
            let key = key ^ u32::from_le_bytes(command.server_time.to_le_bytes());
            command.buttons |= keyed_fields::bit(&mut input, key)?;
            read_angles_and_movement(&mut input, key, baseline, &mut command)?;
        }
    }
    *reader = input;
    Ok(command)
}

fn read_full(
    input: &mut MessageReader<'_>,
    key: u32,
    baseline: &LegacyUserCommand,
    old_melee_angle: u16,
    command: &mut LegacyUserCommand,
) -> Result<(), UserCommandError> {
    command.buttons &= 1;
    command.angles[2] = keyed_fields::delta_short(input, key, baseline.angles[2])?;
    command.buttons |= 2 * keyed_fields::delta_bits(input, key, baseline.buttons >> 1, 20)?;
    command.weapon = u8::try_from(keyed_fields::delta_bits(
        input,
        key,
        u32::from(baseline.weapon),
        7,
    )?)
    .map_err(|_| UserCommandError::InvalidBaseline)?;
    command.offhand = u8::try_from(keyed_fields::delta_bits(
        input,
        key,
        u32::from(baseline.offhand),
        7,
    )?)
    .map_err(|_| UserCommandError::InvalidBaseline)?;
    if command.buttons & 0x10000 != 0 {
        for (selected, old) in command
            .selected_location
            .iter_mut()
            .zip(baseline.selected_location)
        {
            *selected = keyed_fields::delta_byte(input, key, old)?;
        }
    }
    if command.buttons & 4 != 0 {
        command.melee_charge_yaw =
            melee_angle::decode(keyed_fields::delta_short(input, key, old_melee_angle)?);
        command.melee_charge_distance = u8::try_from(keyed_fields::delta_bits(
            input,
            key,
            u32::from(baseline.melee_charge_distance),
            8,
        )?)
        .map_err(|_| UserCommandError::InvalidBaseline)?;
    }
    Ok(())
}

fn read_angles_and_movement(
    input: &mut MessageReader<'_>,
    key: u32,
    baseline: &LegacyUserCommand,
    command: &mut LegacyUserCommand,
) -> Result<(), MessageError> {
    command.angles[0] = keyed_fields::delta_short(input, key, baseline.angles[0])?;
    command.angles[1] = keyed_fields::delta_short(input, key, baseline.angles[1])?;
    let old_mask = axis_mask(baseline.forward_move) | (axis_mask(baseline.right_move) << 2);
    let mask = keyed_fields::delta_bits(input, key, old_mask, 4)?;
    command.forward_move = decode_axis(mask);
    command.right_move = decode_axis(mask >> 2);
    Ok(())
}

fn axis_mask(axis: i8) -> u32 {
    if axis > 10 {
        1
    } else if axis < -10 {
        2
    } else {
        0
    }
}

fn decode_axis(mask: u32) -> i8 {
    if mask & 1 != 0 {
        127
    } else if mask & 2 != 0 {
        -127
    } else {
        0
    }
}
