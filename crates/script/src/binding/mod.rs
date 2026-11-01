// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded local function/developer binding and legacy names, without runtime IDs.

mod legacy_developer_mode;
mod legacy_function_binding;
mod legacy_identifier;
mod legacy_identifier_error;
mod legacy_module_symbols;
mod script_paths;
mod symbol_budget;
mod symbol_builder;
mod symbol_error;
mod symbol_ledger;

pub use legacy_developer_mode::LegacyDeveloperMode;
pub use legacy_function_binding::{
    LegacyFunctionBinding, LegacyFunctionInclusion, LegacyFunctionVisibility,
};
pub use legacy_identifier::LegacyIdentifier;
pub use legacy_identifier_error::LegacyIdentifierError;
pub use legacy_module_symbols::{LegacyModuleSymbols, LegacySymbolUsage};
pub use script_paths::{
    CanonicalScriptPath, CanonicalScriptPathBytes, LegacyScriptPath, ScriptPathBudget,
    ScriptPathError, ScriptPathErrorKind, ScriptPathResource, ScriptPathResult, ScriptPathUsage,
};
pub use symbol_budget::SymbolBudget;
pub use symbol_error::{SymbolError, SymbolErrorKind, SymbolResource};
