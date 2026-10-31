// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable local function-index admission, independent of native string IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SymbolBudget {
    /// Function definitions in this module, including developer-only definitions.
    pub max_functions: u64,
    /// Total borrowed function-name bytes, admitted before name scanning/hashing.
    pub max_name_bytes: u64,
    /// One name's bytes; this policy does not determine legacy case semantics.
    pub max_single_name_bytes: u64,
    /// Logical item visits plus admitted name bytes; not elapsed CPU/instructions.
    pub max_steps: u64,
    /// Logical function records and map key/value entries; excludes source/tree/
    /// declarations, spare capacity, hash-table control bytes, and allocator overhead.
    pub max_metadata_bytes: u64,
}

impl Default for SymbolBudget {
    fn default() -> Self {
        Self {
            max_functions: 1_000_000,
            max_name_bytes: 64 * 1024 * 1024,
            max_single_name_bytes: 64 * 1024 * 1024,
            max_steps: 128 * 1024 * 1024,
            max_metadata_bytes: 128 * 1024 * 1024,
        }
    }
}
