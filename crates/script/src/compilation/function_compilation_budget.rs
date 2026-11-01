// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelExecutionBudget, KernelVerificationBudget};
use crate::literal::LegacyLiteralBudget;
use crate::statement::StatementListBudget;

/// Independent resource scopes for the restricted function compilers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionCompilationBudget {
    /// Body/task/group probes, literal/name bytes, snapshot visits and resolution.
    pub max_steps: u64,
    /// Complete emitted primitive instruction count.
    pub max_instructions: u64,
    /// Maximum live iterative lowering frames.
    pub max_task_frames: u64,
    /// Logical pending/task/binding-pair/snapshot payload and overlapping resolved code/maps.
    /// Excludes source/CST, spare vector/hash capacity, hash control bytes, fixed
    /// local-field/allocator overhead, body-index storage and kernel scopes.
    pub max_metadata_bytes: u64,
    /// Bound local names, including parameters; no native local-count ceiling.
    pub max_local_bindings: u64,
    /// Identifier bytes charged across declaration/read/initialization visits.
    pub max_identifier_bytes: u64,
    /// Live initialization snapshot flag payload, also included in metadata bytes.
    pub max_control_bytes: u64,
    /// Separately admitted immediate body index for the local-statements profile.
    pub body: StatementListBudget,
    /// Per-token conversion limits; strings are unsupported before decoding.
    pub literal: LegacyLiteralBudget,
    /// Separately admitted owned kernel proof and its temporary verification slots.
    pub verification: KernelVerificationBudget,
    /// Separately admitted pure-return validation execution, including stack/fuel;
    /// the local-statements profile does not pre-execute argument-dependent code.
    pub validation: KernelExecutionBudget,
}

impl Default for FunctionCompilationBudget {
    fn default() -> Self {
        Self {
            max_steps: 16_000_000,
            max_instructions: 1_000_000,
            max_task_frames: 1_000_000,
            max_metadata_bytes: 256 * 1024 * 1024,
            max_local_bindings: 1_000_000,
            max_identifier_bytes: 128 * 1024 * 1024,
            max_control_bytes: 128 * 1024 * 1024,
            body: StatementListBudget::default(),
            literal: LegacyLiteralBudget::default(),
            verification: KernelVerificationBudget::default(),
            validation: KernelExecutionBudget::default(),
        }
    }
}
