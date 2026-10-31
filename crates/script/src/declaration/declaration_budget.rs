// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable declaration extraction limits, independent of native compiler pools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclarationBudget {
    /// Total top-level items, including function/directive/developer markers.
    pub max_items: u64,
    /// Include directives, preserving duplicates and source order.
    pub max_includes: u64,
    /// Total parameter declarations across functions, preserving spelling/order.
    pub max_parameters: u64,
    /// Node visits, including scans and parameter-list lowering.
    pub max_steps: u64,
    /// Logical retained item/include/parameter storage; excludes tree/source,
    /// vector spare capacity, allocator overhead, and stack-local builder fields.
    pub max_metadata_bytes: u64,
}

impl Default for DeclarationBudget {
    fn default() -> Self {
        Self {
            max_items: 1_000_000,
            max_includes: 1_000_000,
            max_parameters: 1_000_000,
            max_steps: 16_000_000,
            max_metadata_bytes: 128 * 1024 * 1024,
        }
    }
}
