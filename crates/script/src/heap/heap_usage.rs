// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::StringId;

/// Complete retained store accounting; no allocator rounding/spare capacity/RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StringHeapUsage {
    /// Current live immutable strings.
    pub live_strings: u64,
    /// Current live payload bytes.
    pub payload_bytes: u64,
    /// All retained slots, including vacant/retired records.
    pub slots: u64,
    /// Permanently retired generation-exhausted slots.
    pub retired_slots: u64,
    /// Logical complete retained slot records.
    pub metadata_bytes: u64,
}

/// Logical work/scratch admission for one completed or failed operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HeapWorkUsage {
    /// Charged visits, admitted copy bytes and possible metadata moves.
    pub steps: u64,
    /// Admitted temporary mark-flag payload, excluding borrowed roots.
    pub scratch_bytes: u64,
}

/// One fully published owned value and its operation accounting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeapAllocation {
    /// Opaque nonowning checked reference.
    pub id: StringId,
    /// Complete successful operation accounting.
    pub usage: HeapWorkUsage,
}

/// Complete successful root-based reclamation; no partial deletion on failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StringCollection {
    /// Released immutable values not present in the complete supplied roots.
    pub freed_strings: u64,
    /// Released logical payload bytes.
    pub freed_payload_bytes: u64,
    /// Complete successful work/scratch admission.
    pub usage: HeapWorkUsage,
}
