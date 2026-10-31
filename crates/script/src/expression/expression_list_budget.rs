// SPDX-License-Identifier: AGPL-3.0-or-later

/// Expression-list lowering budgets, independent of native argument/VM arrays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpressionListBudget {
    /// Complete retained expression count; nested descendants are not flattened.
    pub max_expressions: u64,
    /// Node visits across preflight and collection; element reversal is count-bounded.
    pub max_steps: u64,
    /// Logical retained scoped-node bytes; excludes source/CST, spare capacity,
    /// allocator overhead, and stack-local builder state.
    pub max_metadata_bytes: u64,
}

impl Default for ExpressionListBudget {
    fn default() -> Self {
        Self {
            max_expressions: 1_000_000,
            max_steps: 16_000_000,
            max_metadata_bytes: 128 * 1024 * 1024,
        }
    }
}
