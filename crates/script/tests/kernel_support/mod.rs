// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::execution::{KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;

pub fn integer(value: i32) -> KernelInstruction {
    KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Integer(value)))
}

pub fn number(kind: &str, bits: &str) -> Result<LegacyNumber, String> {
    let bits = u32::from_str_radix(bits, 16).map_err(|_| "Invalid scalar fixture bits")?;
    match kind {
        "I" => Ok(LegacyNumber::Integer(bits.cast_signed())),
        "F" => Ok(LegacyNumber::Float(f32::from_bits(bits))),
        _ => Err("Unknown scalar fixture kind".to_owned()),
    }
}
