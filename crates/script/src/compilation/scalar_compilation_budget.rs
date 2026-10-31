// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelExecutionBudget, KernelVerificationBudget};
use crate::literal::LegacyLiteralBudget;

/// Independent resource scopes for the restricted pure scalar function compiler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScalarCompilationBudget {
    /// Body probe/task actions, numeric literal bytes, and instruction resolution.
    pub max_steps: u64,
    /// Complete emitted scalar instruction count.
    pub max_instructions: u64,
    /// Maximum live iterative lowering frames.
    pub max_task_frames: u64,
    /// Logical pending/task and overlapping resolved code/map bytes; excludes
    /// source/CST, vector spare capacity, allocator overhead, and kernel scopes.
    pub max_metadata_bytes: u64,
    /// Per-token conversion limits; strings are unsupported before decoding.
    pub literal: LegacyLiteralBudget,
    /// Separately admitted owned kernel proof and its temporary verification slots.
    pub verification: KernelVerificationBudget,
    /// Separately admitted pure validation execution, including stack and fuel.
    pub validation: KernelExecutionBudget,
}

impl Default for ScalarCompilationBudget {
    fn default() -> Self {
        Self {
            max_steps: 16_000_000,
            max_instructions: 1_000_000,
            max_task_frames: 1_000_000,
            max_metadata_bytes: 256 * 1024 * 1024,
            literal: LegacyLiteralBudget::default(),
            verification: KernelVerificationBudget::default(),
            validation: KernelExecutionBudget::default(),
        }
    }
}
