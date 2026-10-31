// SPDX-License-Identifier: AGPL-3.0-or-later
/// Copyable ownership statistics suitable for aggregate runtime metrics.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct CatalogUsage {
    /// Currently registered assets.
    pub assets: usize,
    /// Initialized metadata slots, including recyclable and retired slots.
    pub slots: usize,
    /// Currently catalog-owned encoded payload bytes.
    pub payload_bytes: u64,
    /// Identifier bytes in both the ownership and lookup copies.
    pub name_bytes: u64,
    /// Maximum simultaneous catalog-owned encoded payload bytes observed.
    pub peak_payload_bytes: u64,
}
