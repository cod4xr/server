// SPDX-License-Identifier: AGPL-3.0-or-later
use super::CompressionError;
use super::symbol_decoder;
use crate::MAX_MESSAGE_BYTES;

/// Decodes a complete byte-framed legacy Huffman payload.
///
/// Upstream continues starting symbols while physical input bits remain; the
/// last symbol may extend past the byte boundary. This implementation supplies
/// virtual zero padding for that symbol without reading memory out of bounds.
/// Extra decoded padding symbols are preserved; a higher-level EOF opcode is
/// required to determine actual message contents. This is not an integrity check.
///
/// A bounded first pass validates all symbols and the output length before a
/// second pass writes output. Errors leave output entirely unchanged.
///
/// # Errors
/// Rejects profile overflow, insufficient output, and reserved/invalid codes.
pub fn decompress(input: &[u8], output: &mut [u8]) -> Result<usize, CompressionError> {
    if input.len() > MAX_MESSAGE_BYTES {
        return Err(CompressionError::MessageTooLarge);
    }
    let total_bits = input.len() * 8;
    let mut position = 0;
    let mut length = 0;
    while position < total_bits {
        symbol_decoder::decode(input, &mut position)?;
        length += 1;
        if length > MAX_MESSAGE_BYTES {
            return Err(CompressionError::MessageTooLarge);
        }
        if length > output.len() {
            return Err(CompressionError::OutputCapacity);
        }
    }
    position = 0;
    for byte in &mut output[..length] {
        // The immutable input and constant tree were validated in the first pass.
        *byte = symbol_decoder::decode(input, &mut position)?;
    }
    Ok(length)
}
