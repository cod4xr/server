// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable encoded/decoded admission, separate from engine storage pools.
///
/// These are logical byte budgets, not allocator overhead or process RSS quotas.
/// No stream blocks or external images are allocated by envelope validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FastFileBudget {
    /// Entire encoded file length, including its twelve-byte outer header.
    pub max_file_bytes: u64,
    /// Declared decoded payload, excluding the fixed `XFile` header.
    pub max_payload_bytes: u64,
    /// A single declared legacy stream block's allocation size.
    pub max_block_bytes: u64,
    /// Sum of all declared stream block allocation sizes.
    pub max_total_block_bytes: u64,
    /// Declared external data accounting; external assets are not read here.
    pub max_external_bytes: u64,
}

impl Default for FastFileBudget {
    fn default() -> Self {
        Self {
            max_file_bytes: 2 * 1024 * 1024 * 1024,
            max_payload_bytes: 2 * 1024 * 1024 * 1024,
            max_block_bytes: 1024 * 1024 * 1024,
            max_total_block_bytes: 2 * 1024 * 1024 * 1024,
            max_external_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}
