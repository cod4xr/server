// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::fastfile::{FastFileHeader, IW3_BLOCK_COUNT};
use crate::zone::ZonePointerError;

/// Logical serialized block address, never a host pointer or engine asset ID.
///
/// The legacy representation uses a four-bit block tag and 28-bit offset after
/// subtracting one. Widened arithmetic does not widen that serialized profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneAddress {
    block: u8,
    offset: u64,
}

impl ZoneAddress {
    /// Returns the IW3 profile's block index, not an engine allocation namespace.
    #[must_use]
    pub fn block(self) -> u8 {
        self.block
    }

    /// Returns byte offset relative to that serialized block.
    #[must_use]
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Checks one logical span against declared block capacity without reading it.
    ///
    /// Zero-length spans may end exactly at capacity. This does not prove bytes
    /// were loaded, an alias was bound, the target type/alignment is valid, or the
    /// address belongs to any particular zone; callers retain that ownership.
    ///
    /// # Errors
    /// Returns a size overflow or declared-block bounds error. No memory is touched.
    pub fn validate_span(self, header: FastFileHeader, bytes: u64) -> Result<(), ZonePointerError> {
        let end = self
            .offset
            .checked_add(bytes)
            .ok_or(ZonePointerError::SizeOverflow)?;
        if end > header.block_bytes()[usize::from(self.block)] {
            return Err(ZonePointerError::BlockBounds);
        }
        Ok(())
    }

    pub(super) fn decode(value: u32) -> Result<Self, ZonePointerError> {
        let block = u8::try_from(value >> 28).map_err(|_| ZonePointerError::InvalidBlock)?;
        if usize::from(block) >= IW3_BLOCK_COUNT {
            return Err(ZonePointerError::InvalidBlock);
        }
        Ok(Self {
            block,
            offset: u64::from(value & 0x0fff_ffff),
        })
    }
}
