// SPDX-License-Identifier: AGPL-3.0-or-later
use super::CompressionError;
use super::codebook::{CODES, WIDTHS};
use crate::MAX_MESSAGE_BYTES;

/// Encodes legacy static Huffman bytes, returning the complete output length.
///
/// Codes are emitted least-significant-bit first and unused final bits are zero.
/// Size/capacity validation finishes before output changes. Input and encoded
/// output are each bounded by the current compatibility message profile; these
/// are wire limits, not engine asset or memory budgets.
///
/// # Errors
/// Returns [`CompressionError::MessageTooLarge`] for profile overflow, or
/// [`CompressionError::OutputCapacity`] when output does not fit.
pub fn compress(input: &[u8], output: &mut [u8]) -> Result<usize, CompressionError> {
    if input.len() > MAX_MESSAGE_BYTES {
        return Err(CompressionError::MessageTooLarge);
    }
    let bits: usize = input
        .iter()
        .map(|&byte| usize::from(WIDTHS[usize::from(byte)]))
        .sum();
    let length = bits.div_ceil(8);
    if length > MAX_MESSAGE_BYTES {
        return Err(CompressionError::MessageTooLarge);
    }
    if output.len() < length {
        return Err(CompressionError::OutputCapacity);
    }
    output[..length].fill(0);
    let mut position = 0;
    for &byte in input {
        let index = usize::from(byte);
        let code = CODES[index];
        for shift in 0..WIDTHS[index] {
            if (code >> shift) & 1 != 0 {
                output[position / 8] |= 1 << (position % 8);
            }
            position += 1;
        }
    }
    Ok(length)
}
