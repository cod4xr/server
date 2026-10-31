// SPDX-License-Identifier: AGPL-3.0-or-later
/// Logical raw-file-only decode usage; no names, payloads, or native addresses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawFileUsage {
    /// Root directory slots, including nulls and aliases.
    pub root_entries: usize,
    /// Newly read name bytes including NULs; prefix names/references are not reread.
    pub inline_name_bytes: u64,
    /// Newly read buffer bytes including final NULs; aliases are not reread.
    pub inline_buffer_bytes: u64,
    /// Peak logical entry/alias/name-span metadata; staging is released on success.
    ///
    /// Excludes separately budgeted prefix metadata, spare capacity, hash/allocator
    /// overhead, and owned source bytes. This is not retained memory or RSS.
    pub peak_metadata_bytes: u64,
    /// Virtual block-four usage including prefix/alignment/inserted alias slots.
    pub virtual_bytes: u64,
    /// High-water temporary block-zero usage; scoped headers reuse that space.
    pub temporary_bytes: u64,
}
