// SPDX-License-Identifier: AGPL-3.0-or-later

/// Program admission limits, independent of native code/stack capacities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelVerificationBudget {
    /// All input instructions, including unreachable code.
    pub max_instructions: u64,
    /// Operand scans, reachable instruction visits, and propagated edges.
    pub max_steps: u64,
    /// Maximum reachable stack depth across the control-flow proof.
    pub max_stack_values: u64,
    /// Logical code plus full state/worklist slots admitted before allocation;
    /// excludes input storage, allocator rounding, local fields, and process RSS.
    pub max_metadata_bytes: u64,
}

impl Default for KernelVerificationBudget {
    fn default() -> Self {
        Self {
            max_instructions: 1_000_000,
            max_steps: 8_000_000,
            max_stack_values: 1_000_000,
            max_metadata_bytes: 256 * 1024 * 1024,
        }
    }
}

/// Per-invocation pure execution limits; fuel is instruction work, not wall time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelExecutionBudget {
    /// Executed instructions, including the return instruction.
    pub max_steps: u64,
    /// Admitted verified maximum stack depth before allocation.
    pub max_stack_values: u64,
    /// Logical reserved value slots; excludes allocator rounding/local fields/RSS.
    pub max_stack_bytes: u64,
}

impl Default for KernelExecutionBudget {
    fn default() -> Self {
        Self {
            max_steps: 1_000_000,
            max_stack_values: 1_000_000,
            max_stack_bytes: 64 * 1024 * 1024,
        }
    }
}
