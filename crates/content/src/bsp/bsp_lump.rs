// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::slice::ChunksExact;

use crate::bsp::{BspError, LumpId};

/// One validated borrowed payload with its original container identity/offset.
#[derive(Clone, Copy)]
pub struct BspLump<'file> {
    pub(super) id: LumpId,
    pub(super) offset: u64,
    pub(super) bytes: &'file [u8],
}

impl<'file> BspLump<'file> {
    /// Returns the original raw kind, including unknown/duplicate kinds.
    #[must_use]
    pub fn id(self) -> LumpId {
        self.id
    }

    /// Returns the original or sequentially computed logical payload offset.
    ///
    /// Empty lumps have no accessible payload and may carry an out-of-file offset.
    #[must_use]
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Returns the exact encoded payload, excluding alignment padding.
    #[must_use]
    pub fn as_bytes(self) -> &'file [u8] {
        self.bytes
    }

    /// Divides the entire payload into equal-size encoded records without casting.
    ///
    /// Each record still needs semantic/endian validation. Empty lumps produce
    /// zero records; this method never invents an empty record or drops a tail.
    ///
    /// # Errors
    /// Rejects zero element size and incomplete final records.
    pub fn records(self, element_bytes: usize) -> Result<ChunksExact<'file, u8>, BspError> {
        if element_bytes == 0 || !self.bytes.len().is_multiple_of(element_bytes) {
            return Err(BspError::InvalidElementSize);
        }
        Ok(self.bytes.chunks_exact(element_bytes))
    }
}

impl fmt::Debug for BspLump<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BspLump")
            .field("id", &self.id)
            .field("offset", &self.offset)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
