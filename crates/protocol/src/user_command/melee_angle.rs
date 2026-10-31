// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::user_command::UserCommandError;

/// Matches the pinned ANGLE2SHORT macro's binary32 multiply/divide, followed by
/// truncation to signed int and masking. Reject C's undefined conversion domain.
#[allow(
    clippy::cast_possible_truncation,
    reason = "Legacy ANGLE2SHORT truncates a range-checked float."
)]
pub(super) fn encode(yaw: f32) -> Result<u16, UserCommandError> {
    let scaled = yaw * 65536.0_f32 / 360.0_f32;
    if !scaled.is_finite() || !(-2_147_483_648.0_f32..2_147_483_648.0_f32).contains(&scaled) {
        return Err(UserCommandError::InvalidBaseline);
    }
    u16::try_from((scaled as i32) & 65535).map_err(|_| UserCommandError::InvalidBaseline)
}

pub(super) fn decode(bits: u16) -> f32 {
    // The original signed-short conversion is intentional. The double literal
    // rounds every possible short product to the same binary32 value as 360/65536.
    f32::from(i16::from_le_bytes(bits.to_le_bytes())) * (360.0_f32 / 65536.0_f32)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "Test checks every short against an original-C digest."
    )]

    use crate::user_command::melee_angle;

    #[test]
    fn every_signed_short_angle_has_the_original_binary32_result() {
        let expected = include_str!("../../tests/fixtures/user_command_vectors.txt")
            .lines()
            .find_map(|line| line.strip_prefix("# melee_angles_fnv64 "))
            .expect("Original C exhaustive angle digest");
        let expected = u64::from_str_radix(expected, 16).expect("Digest hex");
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for raw in 0..=u16::MAX {
            for byte in melee_angle::decode(raw).to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0100_0000_01b3);
            }
        }
        assert_eq!(hash, expected);
    }
}
