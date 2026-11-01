// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::FunctionLoweringBudget;
use crate::execution::KernelModuleVerificationBudget;

/// Complete bound-module code-generation admission, independent of native pools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModuleCompilationBudget {
    /// All recorded definitions, including disabled developer declarations.
    pub max_declarations: u64,
    /// Active executable definitions and local target-table entries.
    pub max_functions: u64,
    /// Inventory/name/record visits plus all function compiler work.
    pub max_steps: u64,
    /// Total instructions across all staged active functions.
    pub max_instructions: u64,
    /// Logical tables/scope/staged code/maps plus current lowering overlap.
    /// Excludes source/CST/symbols, nested body/literal/kernel scopes, hash controls,
    /// spare capacity, allocator rounding, fixed fields and process RSS.
    pub max_metadata_bytes: u64,
    /// Individual code-generation limits, clamped to remaining global admission.
    pub function: FunctionLoweringBudget,
    /// Separately admitted whole-inventory kernel proof, additional to staged input.
    pub verification: KernelModuleVerificationBudget,
}

impl Default for ModuleCompilationBudget {
    fn default() -> Self {
        Self {
            max_declarations: 1_000_000,
            max_functions: 1_000_000,
            max_steps: 16_000_000,
            max_instructions: 1_000_000,
            max_metadata_bytes: 256 * 1024 * 1024,
            function: FunctionLoweringBudget::default(),
            verification: KernelModuleVerificationBudget::default(),
        }
    }
}

/// Bound-module compiler resource; all but `CallArguments` are aggregate scopes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleCompilationResource {
    /// All retained source declaration records.
    Declarations,
    /// Active local function targets.
    Functions,
    /// Inventory/name/record/code-generation work.
    Steps,
    /// Complete staged instruction count.
    Instructions,
    /// Logical compiler-owned global tables/payload and current lowering overlap.
    MetadataBytes,
    /// Argument expressions at one call site, under the function lowering budget.
    CallArguments,
}
