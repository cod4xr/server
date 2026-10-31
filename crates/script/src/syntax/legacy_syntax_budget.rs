// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::lexical::LegacyLexBudget;

/// Configurable structural parser limits, independent of old native stack arrays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacySyntaxBudget {
    /// Source-scanner work/output limits; source indexing is admitted separately.
    pub lexical: LegacyLexBudget,
    /// Flat tree nodes including selectors, terminals, EOFs, and productions.
    pub max_nodes: u64,
    /// Flat parent-child edges.
    pub max_child_edges: u64,
    /// Live parser stack entries, including the initial state sentinel.
    pub max_stack_entries: u64,
    /// Grammar actions/reductions, bounding empty-reduction loops as well as shifts.
    pub max_steps: u64,
    /// Logical node/edge/live-stack metadata, excluding spare capacity, allocator
    /// overhead, source bytes, and separately admitted source-index storage.
    pub max_metadata_bytes: u64,
}

impl Default for LegacySyntaxBudget {
    fn default() -> Self {
        Self {
            lexical: LegacyLexBudget::default(),
            max_nodes: 1_000_000,
            max_child_edges: 2_000_000,
            max_stack_entries: 1_000_000,
            max_steps: 16_000_000,
            max_metadata_bytes: 256 * 1024 * 1024,
        }
    }
}
