// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::FunctionCompilationBudget;
use crate::literal::LegacyLiteralBudget;
use crate::statement::StatementListBudget;

const DEFAULT_MAX_CALL_ARGUMENTS: u64 = 1_000_000;

/// Function code-generation limits shared by isolated and bound-module compilation.
/// Kernel proof and pure validation have separate admission scopes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionLoweringBudget {
    /// Body/task/group probes, literal/name bytes, snapshots and resolution work.
    pub max_steps: u64,
    /// Complete emitted typed instructions.
    pub max_instructions: u64,
    /// Maximum live iterative lowering tasks.
    pub max_task_frames: u64,
    /// Logical pending/task/binding/snapshot and overlapping resolved code/map payload.
    /// Excludes source/CST, body-index/kernel scopes, hash controls/spare capacity,
    /// allocator rounding, fixed fields and process RSS.
    pub max_metadata_bytes: u64,
    /// Bound parameter/local names.
    pub max_local_bindings: u64,
    /// Identifier bytes across declarations/reads/call resolution/initialization.
    pub max_identifier_bytes: u64,
    /// Live initialization-snapshot flag payload, also counted in metadata.
    pub max_control_bytes: u64,
    /// Supplied argument expressions at any one bound-module call site.
    pub max_call_arguments: u64,
    /// Separately admitted body indexing.
    pub body: StatementListBudget,
    /// Separately admitted per-token literal conversion.
    pub literal: LegacyLiteralBudget,
}

impl Default for FunctionLoweringBudget {
    fn default() -> Self {
        Self {
            max_steps: 16_000_000,
            max_instructions: 1_000_000,
            max_task_frames: 1_000_000,
            max_metadata_bytes: 256 * 1024 * 1024,
            max_local_bindings: 1_000_000,
            max_identifier_bytes: 128 * 1024 * 1024,
            max_control_bytes: 128 * 1024 * 1024,
            max_call_arguments: DEFAULT_MAX_CALL_ARGUMENTS,
            body: StatementListBudget::default(),
            literal: LegacyLiteralBudget::default(),
        }
    }
}

impl From<FunctionCompilationBudget> for FunctionLoweringBudget {
    fn from(budget: FunctionCompilationBudget) -> Self {
        Self {
            max_steps: budget.max_steps,
            max_instructions: budget.max_instructions,
            max_task_frames: budget.max_task_frames,
            max_metadata_bytes: budget.max_metadata_bytes,
            max_local_bindings: budget.max_local_bindings,
            max_identifier_bytes: budget.max_identifier_bytes,
            max_control_bytes: budget.max_control_bytes,
            max_call_arguments: DEFAULT_MAX_CALL_ARGUMENTS,
            body: budget.body,
            literal: budget.literal,
        }
    }
}
