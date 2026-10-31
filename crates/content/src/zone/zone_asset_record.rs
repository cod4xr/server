// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::zone::{ZonePointer, ZonePointerError};

/// Raw IW3 root directory record, separate from a decoded or registered asset.
///
/// Unknown kinds and all encoded header values are preserved. No pointer is
/// followed, authenticated, or converted to a native address by this type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneAssetRecord {
    kind: u32,
    header: u32,
}

impl ZoneAssetRecord {
    /// Returns the raw serialized type, not the engine catalog's asset kind.
    #[must_use]
    pub fn kind(self) -> u32 {
        self.kind
    }

    /// Returns the unresolved raw header, including possible null/sentinel values.
    #[must_use]
    pub fn encoded_header(self) -> u32 {
        self.header
    }

    /// Classifies the header word without resolving an asset or alias target.
    ///
    /// # Errors
    /// Rejects invalid block tags. Type-specific pointer rules remain unchecked.
    pub fn decode_header(self) -> Result<ZonePointer, ZonePointerError> {
        ZonePointer::decode(self.header)
    }

    pub(super) fn parse(bytes: &[u8]) -> Self {
        // Only exact eight-byte records from the validated borrowed table call here.
        Self {
            kind: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            header: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        }
    }
}
