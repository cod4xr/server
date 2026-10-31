// SPDX-License-Identifier: AGPL-3.0-or-later
/// Raw container lump kind, independent of engine entity/asset identifiers.
///
/// Unknown kinds are retained. Semantic decoding must select a version-specific
/// schema instead of treating arbitrary raw IDs as known geometry structures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LumpId(u32);

impl LumpId {
    /// Preserves the full raw directory identifier without narrowing.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact format identifier.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}
