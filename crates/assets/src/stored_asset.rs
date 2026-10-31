// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::AssetKind;

/// Owned opaque encoded content. Its debug representation excludes name/data.
pub struct StoredAsset {
    pub(super) kind: AssetKind,
    pub(super) name: Vec<u8>,
    pub(super) bytes: Vec<u8>,
}

impl StoredAsset {
    /// Returns the engine category, independent of serialized asset type IDs.
    #[must_use]
    pub fn kind(&self) -> AssetKind {
        self.kind
    }

    /// Returns the exact byte identifier; no UTF-8/path interpretation is implied.
    #[must_use]
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    /// Returns encoded bytes without interpreting them as a validated asset format.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for StoredAsset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoredAsset")
            .field("kind", &self.kind)
            .field("name_bytes", &self.name.len())
            .field("payload_bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
