// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;

/// Successful immutable-module proof accounting, excluding inputs/allocator/RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KernelModuleUsage {
    /// Complete immutable literal row count, including unused rows.
    pub string_literals: u64,
    /// Retained literal rows and payload, also included in `retained_bytes`.
    pub retained_string_bytes: u64,
    /// Retained verified function count.
    pub functions: u64,
    /// Retained complete instruction count.
    pub instructions: u64,
    /// Inventory visits and all admitted per-function proof work.
    pub verification_steps: u64,
    /// Complete retained function-table, code and literal row/payload bytes.
    pub retained_bytes: u64,
    /// Peak full retained payload plus one function's verifier temporary payload.
    pub verification_metadata_bytes: u64,
}

/// Aggregate invocation accounting. Counts represent logical reserved payload,
/// excluding allocator rounding, vector spare capacity, fixed fields and RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KernelCallUsage {
    /// Admitted instruction visits across every frame, including a failing operation.
    pub steps: u64,
    /// Admitted conservative frame initialization/copy/reservation work.
    pub setup_work: u64,
    /// Maximum admitted simultaneous frame count, before allocation.
    pub peak_frames: u64,
    /// Maximum admitted simultaneous frame-table payload, before allocation.
    pub peak_frame_bytes: u64,
    /// Maximum admitted simultaneous reserved operands, not touched stack slots.
    pub peak_stack_values: u64,
    /// Maximum admitted simultaneous reserved operand payload bytes.
    pub peak_stack_bytes: u64,
    /// Maximum admitted simultaneous complete local slots.
    pub peak_local_values: u64,
    /// Maximum admitted simultaneous initialized-state/local payload bytes.
    pub peak_local_bytes: u64,
}

/// Completed owned primitive result; Debug omits the returned value's contents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KernelCallOutcome {
    /// Returned copied data; string handles remain in an explicit caller-owned heap.
    pub value: KernelValue,
    /// Global work and maximum live payload over this complete invocation.
    pub usage: KernelCallUsage,
}
