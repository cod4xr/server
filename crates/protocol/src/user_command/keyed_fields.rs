// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{MessageError, MessageReader};

pub(super) fn bit(reader: &mut MessageReader<'_>, key: u32) -> Result<u32, MessageError> {
    Ok((reader.read_bits(1)? ^ key) & 1)
}

pub(super) fn delta_bits(
    reader: &mut MessageReader<'_>,
    key: u32,
    old: u32,
    width: u8,
) -> Result<u32, MessageError> {
    if reader.read_bits(1)? == 0 {
        return Ok(old);
    }
    let value = reader.read_bits(width)?;
    let mask = u32::MAX.checked_shr(32 - u32::from(width)).unwrap_or(0);
    Ok((value ^ key) & mask)
}

pub(super) fn delta_short(
    reader: &mut MessageReader<'_>,
    key: u32,
    old: u16,
) -> Result<u16, MessageError> {
    if reader.read_bits(1)? == 0 {
        return Ok(old);
    }
    let key = key.to_le_bytes();
    let value = reader.read_i16()?.to_le_bytes();
    Ok(u16::from_le_bytes([key[0] ^ value[0], key[1] ^ value[1]]))
}

pub(super) fn delta_byte(
    reader: &mut MessageReader<'_>,
    key: u32,
    old: u8,
) -> Result<u8, MessageError> {
    if reader.read_bits(1)? == 0 {
        return Ok(old);
    }
    Ok(reader.read_u8()? ^ key.to_le_bytes()[0])
}
