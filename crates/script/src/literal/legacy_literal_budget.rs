// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable per-literal admission, separate from source/module/runtime budgets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyLiteralBudget {
    /// Complete encoded lexeme bytes, including literal prefixes/delimiters.
    pub max_encoded_bytes: u64,
    /// Decoded string payload bytes, excluding any implicit C terminator.
    ///
    /// Storage grows fallibly; this does not allocate a fixed compiler scratch
    /// buffer or impose the source implementation's 8191-byte temporary bound.
    pub max_string_bytes: u64,
}

impl Default for LegacyLiteralBudget {
    fn default() -> Self {
        Self {
            max_encoded_bytes: 64 * 1024 * 1024,
            max_string_bytes: 64 * 1024 * 1024,
        }
    }
}
