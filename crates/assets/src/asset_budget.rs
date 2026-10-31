// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable catalog ownership budgets, never legacy client limits.
///
/// Zero disables the corresponding resource. Defaults allocate no payload or
/// metadata eagerly. Byte counters describe owned logical data, not allocator
/// overhead, decoded asset memory, GPU residency, or total process RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssetBudget {
    /// Maximum allocated metadata slots, including retired generation slots.
    pub max_assets: usize,
    /// Maximum total catalog-owned encoded payload bytes.
    pub max_payload_bytes: u64,
    /// Maximum encoded bytes for one asset.
    pub max_single_asset_bytes: u64,
    /// Maximum bytes in one exact asset identifier; it is not a filesystem path.
    pub max_name_bytes: usize,
    /// Total stored name bytes, accounting for both ownership/index copies.
    pub max_total_name_bytes: u64,
}

impl Default for AssetBudget {
    fn default() -> Self {
        Self {
            max_assets: 100_000,
            max_payload_bytes: 2 * 1024 * 1024 * 1024,
            max_single_asset_bytes: 256 * 1024 * 1024,
            max_name_bytes: 4096,
            max_total_name_bytes: 32 * 1024 * 1024,
        }
    }
}
