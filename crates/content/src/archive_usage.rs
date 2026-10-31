// SPDX-License-Identifier: AGPL-3.0-or-later
/// Admission statistics, independent of decoding, GPU residency, and process RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct ArchiveUsage {
    /// Original directory entries, including directories and duplicates.
    pub entries: usize,
    /// Visible files after case/separator aliases and duplicates are resolved.
    pub files: usize,
    /// Caller-owned encoded storage length.
    pub archive_bytes: u64,
    /// Sum of declared compressed payload lengths, including repeated entries.
    pub compressed_bytes: u64,
    /// Directory-through-footer bytes.
    pub metadata_bytes: u64,
    /// Sum of original directory filename byte lengths.
    pub name_bytes: u64,
    /// Sum of declared output lengths, including overridden entries.
    pub declared_bytes: u64,
}
