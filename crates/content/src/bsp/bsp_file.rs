// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crc32fast::Hasher;

use crate::bsp::bsp_index;
use crate::bsp::{BspBudget, BspError, BspLump, LumpId};

/// Container layout supported by the pinned BSP directory reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BspLayout {
    /// Versions 6..18: a fixed per-version directory of length/offset pairs.
    OffsetDirectory,
    /// Versions 19..45: counted type/length entries and aligned sequential data.
    SequentialChunks,
}

/// Immutable IBSP container with instance-owned, budgeted metadata.
///
/// This validates storage structure, not collision, rendering, map entities, or
/// gameplay compatibility. No global current-map pointer or fixed 100-lump array
/// is retained. Unknown/duplicate kinds remain accessible in directory order.
pub struct BspFile<'file> {
    bytes: &'file [u8],
    version: u32,
    layout: BspLayout,
    lumps: Vec<BspLump<'file>>,
}

impl<'file> BspFile<'file> {
    /// Validates every directory/payload span before publishing a borrowed map.
    ///
    /// Uses checked wide offsets and fallible metadata allocation. Source bytes
    /// remain caller-owned and unchanged; no payload or filename is copied.
    ///
    /// # Errors
    /// Rejects unsupported/truncated/overlapping directory data, exceeded budgets,
    /// arithmetic overflow, or allocation failure without returning partial state.
    pub fn parse(bytes: &'file [u8], budget: BspBudget) -> Result<Self, BspError> {
        let (version, layout, lumps) = bsp_index::parse(bytes, budget)?;
        Ok(Self {
            bytes,
            version,
            layout,
            lumps,
        })
    }

    /// Returns the exact format version without claiming a decoded map schema.
    #[must_use]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns the interpreted directory layout.
    #[must_use]
    pub fn layout(&self) -> BspLayout {
        self.layout
    }

    /// Borrows every directory entry, including empty/unknown/duplicate entries.
    #[must_use]
    pub fn lumps(&self) -> &[BspLump<'file>] {
        &self.lumps
    }

    /// Finds the first matching entry, preserving original BSP lookup precedence.
    ///
    /// A present empty entry is distinct from an absent kind in this metadata API.
    #[must_use]
    pub fn lump(&self, id: LumpId) -> Option<BspLump<'file>> {
        self.lumps.iter().find(|lump| lump.id == id).copied()
    }

    /// Computes the original seeded CRC32 over the entire source, including padding
    /// and trailing bytes. The map checksum uses feed zero. This is not authentication.
    #[must_use]
    pub fn legacy_checksum(&self, feed: u32) -> u32 {
        let mut hasher = Hasher::new_with_initial(feed);
        hasher.update(self.bytes);
        hasher.finalize()
    }
}

impl fmt::Debug for BspFile<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BspFile")
            .field("version", &self.version)
            .field("layout", &self.layout)
            .field("lumps", &self.lumps.len())
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
