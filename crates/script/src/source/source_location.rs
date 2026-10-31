// SPDX-License-Identifier: AGPL-3.0-or-later
/// One-based byte location relative to the source used for the lookup.
///
/// Columns count raw bytes, not Unicode characters, graphemes, or tab-expanded
/// screen cells. A future module/source registry supplies file identity separately.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    /// Zero-based byte offset, including the EOF position.
    pub offset: u64,
    /// One-based line number.
    pub line: u64,
    /// One-based byte column within that line.
    pub byte_column: u64,
}
