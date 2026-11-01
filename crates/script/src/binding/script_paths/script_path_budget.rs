// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable virtual-name canonicalization quotas, not native filename ceilings.
#[derive(Clone, Copy, Debug)]
pub struct ScriptPathBudget {
    /// Complete borrowed input bytes, before canonicalization.
    pub max_input_bytes: u64,
    /// Complete canonical payload bytes, excluding a native terminator.
    pub max_canonical_bytes: u64,
    /// Admitted validation/scanning/output byte visits for one operation.
    pub max_steps: u64,
}

impl Default for ScriptPathBudget {
    fn default() -> Self {
        Self {
            max_input_bytes: 1024 * 1024,
            max_canonical_bytes: 1024 * 1024,
            max_steps: 8 * 1024 * 1024,
        }
    }
}

/// Logical operation accounting; fixed fields/spare capacity/allocator/RSS excluded.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScriptPathUsage {
    /// Complete supplied input length, including bytes rejected by this profile.
    pub input_bytes: u64,
    /// Counted/admitted canonical payload length.
    pub canonical_bytes: u64,
    /// Actual admitted/prepaid visits; failed admission remains uncharged.
    pub work_steps: u64,
    /// Owned canonical payload published by this operation; borrowed views use zero.
    pub retained_bytes: u64,
}

/// Complete successful result without an implicit filesystem permission or interner ID.
#[derive(Debug)]
pub struct ScriptPathResult<T> {
    /// Validated view or independently owned canonical bytes.
    pub value: T,
    /// Logical admitted operation work/storage.
    pub usage: ScriptPathUsage,
}
