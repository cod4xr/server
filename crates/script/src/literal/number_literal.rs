// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::legacy::LegacyNumber;
use crate::literal::LegacyLiteralError;

pub(super) fn integer(bytes: &[u8]) -> Result<LegacyNumber, LegacyLiteralError> {
    if bytes.is_empty() {
        return Err(LegacyLiteralError::InvalidSpelling);
    }
    let mut value = 0_i32;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return Err(LegacyLiteralError::InvalidSpelling);
        }
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(i32::from(byte - b'0')))
            .ok_or(LegacyLiteralError::NumberOutOfRange)?;
    }
    Ok(LegacyNumber::Integer(value))
}

pub(super) fn float(bytes: &[u8]) -> Result<LegacyNumber, LegacyLiteralError> {
    let text = std::str::from_utf8(bytes).map_err(|_| LegacyLiteralError::InvalidSpelling)?;
    let value: f32 = text
        .parse()
        .map_err(|_| LegacyLiteralError::InvalidSpelling)?;
    if !value.is_finite() {
        return Err(LegacyLiteralError::NumberOutOfRange);
    }
    Ok(LegacyNumber::Float(value))
}
