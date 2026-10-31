// SPDX-License-Identifier: AGPL-3.0-or-later
/// Logical root-metadata budgets, independently configurable from legacy handles.
///
/// Borrowed strings/records do not copy payloads. Metadata accounts for the owned
/// string-slot vector, excluding allocator overhead and the separately owned stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZonePrefixBudget {
    /// Number of script-string slots, including null/unresolved slots.
    pub max_script_strings: u64,
    /// Number of root asset records, including unknown types and repeated headers.
    pub max_assets: u64,
    /// One inline string's encoded byte length, including NUL.
    pub max_string_bytes: u64,
    /// Sum of inline string byte lengths, including each NUL.
    pub max_total_string_bytes: u64,
    /// Logical owned string-slot metadata bytes.
    pub max_metadata_bytes: u64,
}

impl Default for ZonePrefixBudget {
    fn default() -> Self {
        Self {
            max_script_strings: 1_000_000,
            max_assets: 1_000_000,
            max_string_bytes: 64 * 1024,
            max_total_string_bytes: 32 * 1024 * 1024,
            max_metadata_bytes: 64 * 1024 * 1024,
        }
    }
}
