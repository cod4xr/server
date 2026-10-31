// SPDX-License-Identifier: AGPL-3.0-or-later
use super::CompressionError;
use super::codebook::TREE;

pub(super) fn decode(input: &[u8], position: &mut usize) -> Result<u8, CompressionError> {
    let mut node = 0;
    // Original codes are at most eleven bits, including the reserved NYT leaf.
    // Virtual zero bits make only the final overread deterministic and safe.
    for _ in 0..11 {
        let bit = input
            .get(*position / 8)
            .map_or(0, |byte| (byte >> (*position % 8)) & 1);
        *position += 1;
        let child = TREE[node][usize::from(bit)];
        if child >= 256 {
            return u8::try_from(child - 256).map_err(|_| CompressionError::ReservedSymbol);
        }
        node = usize::from(child);
    }
    Err(CompressionError::InvalidCode)
}
