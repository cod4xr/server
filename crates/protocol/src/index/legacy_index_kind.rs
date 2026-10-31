// SPDX-License-Identifier: AGPL-3.0-or-later
/// Legacy wire index namespaces. These are never engine-wide allocation limits.
///
/// Wider internal entity/player IDs require an explicit per-peer projection or
/// a negotiated extended codec; narrowing a wider engine ID is not allowed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyIndexKind {
    /// Ten-bit entity indices, including reserved world/none values.
    Entity,
    /// Six-bit client indices in the audited 64-client profile.
    Client,
}

impl LegacyIndexKind {
    /// Returns the largest representable index, including reserved sentinel IDs.
    #[must_use]
    pub fn maximum_index(self) -> u16 {
        (1_u16 << self.bits()) - 1
    }

    pub(super) fn bits(self) -> u8 {
        match self {
            Self::Entity => 10,
            Self::Client => 6,
        }
    }
}
