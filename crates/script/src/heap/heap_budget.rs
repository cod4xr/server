// SPDX-License-Identifier: AGPL-3.0-or-later

/// Retained owned string-store admission, independent of native ID/pool widths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StringHeapBudget {
    /// Live immutable string values, including empty payloads.
    pub max_live_strings: u64,
    /// Sum of live logical payload bytes, without implicit terminators.
    pub max_payload_bytes: u64,
    /// Complete retained slot count, including vacant and retired slots.
    pub max_slots: u64,
    /// Logical slot records; excludes payload, allocator/spare capacity/fixed fields.
    pub max_metadata_bytes: u64,
    /// One immutable payload's complete byte length.
    pub max_single_string_bytes: u64,
}

impl Default for StringHeapBudget {
    fn default() -> Self {
        Self {
            max_live_strings: 1_000_000,
            max_payload_bytes: 256 * 1024 * 1024,
            max_slots: 1_000_000,
            max_metadata_bytes: 128 * 1024 * 1024,
            max_single_string_bytes: 16 * 1024 * 1024,
        }
    }
}

/// One operation's admitted work/scratch, supplied from a caller's remaining budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeapWorkBudget {
    /// Logical slot/root visits, admitted payload copy bytes and possible slot moves.
    /// Allocation failures retain admitted work; this is not measured CPU time.
    pub max_steps: u64,
    /// Logical mark-flag bytes, excluding borrowed roots/fixed fields/allocator overhead.
    pub max_scratch_bytes: u64,
}

impl Default for HeapWorkBudget {
    fn default() -> Self {
        Self {
            max_steps: 64_000_000,
            max_scratch_bytes: 64 * 1024 * 1024,
        }
    }
}
