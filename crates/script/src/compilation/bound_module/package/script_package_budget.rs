// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::ScriptPathBudget;
use crate::compilation::ModuleCompilationBudget;

/// Closed-package namespace and shared code-generation admission.
/// Caller source/CST/symbol storage and kernel proof remain separate scopes.
#[derive(Clone, Copy, Debug)]
pub struct ScriptPackageBudget {
    /// Supplied module records, including empty/all-disabled modules.
    pub max_modules: u64,
    /// Complete raw catalog-name bytes before canonicalization.
    pub max_path_input_bytes: u64,
    /// Supplied include directives across all modules, including empty targets.
    pub max_include_edges: u64,
    /// Imported function-name aliases, additional to direct executable functions.
    pub max_imported_functions: u64,
    /// Per-name validation/copy/query limits, independent of native wrapper bounds.
    pub path: ScriptPathBudget,
    /// Global compiler steps/metadata/instructions/functions across the package.
    /// Namespace paths/rows/sort/comparisons share these compiler bounds.
    pub compilation: ModuleCompilationBudget,
}

impl Default for ScriptPackageBudget {
    fn default() -> Self {
        Self {
            max_modules: 16_384,
            max_path_input_bytes: 64 * 1024 * 1024,
            max_include_edges: 1_000_000,
            max_imported_functions: 1_000_000,
            path: ScriptPathBudget::default(),
            compilation: ModuleCompilationBudget::default(),
        }
    }
}
