// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable container admission budgets, not decoded engine capacity limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BspBudget {
    /// Maximum caller-owned encoded source length.
    pub max_file_bytes: u64,
    /// Maximum directory entries; the original `chunks[100]` is not a Rust ceiling.
    pub max_lumps: usize,
    /// Maximum payload length of a single lump.
    pub max_lump_bytes: u64,
    /// Sum of declared lengths, including duplicate/overlapping old-format spans.
    pub max_total_lump_bytes: u64,
}

impl Default for BspBudget {
    fn default() -> Self {
        Self {
            max_file_bytes: 2 * 1024 * 1024 * 1024,
            max_lumps: 4096,
            max_lump_bytes: 512 * 1024 * 1024,
            max_total_lump_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}
