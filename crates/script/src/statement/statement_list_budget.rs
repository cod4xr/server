// SPDX-License-Identifier: AGPL-3.0-or-later

/// Complete body-list admission limits, independent of native local/VM capacities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatementListBudget {
    /// Retained immediate statements/labels; nested bodies remain separate.
    pub max_statements: u64,
    /// Actual spine visits across preflight/collection; reversal is count-bounded.
    pub max_steps: u64,
    /// Logical retained scoped-node bytes; excludes source/CST, spare capacity,
    /// allocator overhead, and local builder state.
    pub max_metadata_bytes: u64,
}

impl Default for StatementListBudget {
    fn default() -> Self {
        Self {
            max_statements: 1_000_000,
            max_steps: 16_000_000,
            max_metadata_bytes: 128 * 1024 * 1024,
        }
    }
}
