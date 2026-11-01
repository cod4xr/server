// SPDX-License-Identifier: AGPL-3.0-or-later

/// Aggregate immutable-module admission; stack/local limits apply per function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelModuleVerificationBudget {
    /// Complete inventory count, independent of native function namespaces.
    pub max_functions: u64,
    /// Total instructions across every function, including unreachable code.
    pub max_instructions: u64,
    /// Inventory visits plus all function operand/flow proof visits and edges.
    pub max_steps: u64,
    /// Full retained code/table plus current verifier payload; excludes inputs,
    /// allocator rounding, vector spare capacity, fixed fields and process RSS.
    pub max_metadata_bytes: u64,
    /// Maximum admitted reachable stack depth of any individual function.
    pub max_function_stack_values: u64,
    /// Maximum complete local layout of any individual function.
    pub max_function_local_values: u64,
}

impl Default for KernelModuleVerificationBudget {
    fn default() -> Self {
        Self {
            max_functions: 1_000_000,
            max_instructions: 1_000_000,
            max_steps: 8_000_000,
            max_metadata_bytes: 256 * 1024 * 1024,
            max_function_stack_values: 1_000_000,
            max_function_local_values: 1_000_000,
        }
    }
}

/// One complete iterative invocation's global admission and work limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelCallBudget {
    /// Instructions across every called function, including calls and returns.
    pub max_steps: u64,
    /// Conservative frame setup work: one plus local slots, supplied arguments
    /// and prior live frames per admitted frame. Reservation relocation is covered.
    pub max_setup_work: u64,
    /// Simultaneously live frames, including the entry frame.
    pub max_frames: u64,
    /// Logical live frame-table payload, excluding allocator rounding/spare capacity.
    pub max_frame_bytes: u64,
    /// Sum of verified reserved operand slots across all live frames.
    pub max_stack_values: u64,
    /// Logical live reserved primitive operand payload.
    pub max_stack_bytes: u64,
    /// Sum of complete local layouts across all live frames.
    pub max_local_values: u64,
    /// Logical live initialized-state/local payload.
    pub max_local_bytes: u64,
}

impl Default for KernelCallBudget {
    fn default() -> Self {
        Self {
            max_steps: 1_000_000,
            max_setup_work: 16_000_000,
            max_frames: 16_384,
            max_frame_bytes: 16 * 1024 * 1024,
            max_stack_values: 1_000_000,
            max_stack_bytes: 64 * 1024 * 1024,
            max_local_values: 1_000_000,
            max_local_bytes: 64 * 1024 * 1024,
        }
    }
}
