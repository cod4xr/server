// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable source/index admission, independent of legacy compiler arrays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceBudget {
    /// Complete borrowed source length; no implicit truncation or UTF-8 repair.
    pub max_source_bytes: u64,
    /// Indexed lines, including the empty final line after a trailing LF.
    pub max_lines: u64,
    /// Logical owned line-index bytes, excluding allocator overhead/source storage.
    pub max_metadata_bytes: u64,
}

impl Default for SourceBudget {
    fn default() -> Self {
        Self {
            max_source_bytes: 64 * 1024 * 1024,
            max_lines: 1_000_000,
            max_metadata_bytes: 16 * 1024 * 1024,
        }
    }
}
