// SPDX-License-Identifier: AGPL-3.0-or-later
use super::ChallengeError;

pub(super) fn parse(bytes: &[u8]) -> Result<i32, ChallengeError> {
    let (negative, digits) = if let Some(digits) = bytes.strip_prefix(b"-") {
        (true, digits)
    } else {
        (false, bytes)
    };
    if digits.is_empty() {
        return Err(ChallengeError::InvalidInteger);
    }
    let mut value = 0_i32;
    for &digit in digits {
        if !digit.is_ascii_digit() {
            return Err(ChallengeError::InvalidInteger);
        }
        // Accumulate negatively so i32::MIN is admitted without an unsigned cast.
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_sub(i32::from(digit - b'0')))
            .ok_or(ChallengeError::InvalidInteger)?;
    }
    if negative {
        Ok(value)
    } else {
        value.checked_neg().ok_or(ChallengeError::InvalidInteger)
    }
}

pub(super) fn encode(value: i32, storage: &mut [u8; 11]) -> &[u8] {
    let mut magnitude = value.unsigned_abs();
    let mut start = storage.len();
    loop {
        start -= 1;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "A decimal remainder is at most nine."
        )]
        let digit = (magnitude % 10) as u8;
        storage[start] = b'0' + digit;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        start -= 1;
        storage[start] = b'-';
    }
    &storage[start..]
}
