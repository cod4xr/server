// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed legacy module declarations above structural trees, without name binding.

mod declaration_budget;
mod declaration_error;
mod declaration_ledger;
mod declaration_lowering;
mod legacy_anim_tree_syntax;
mod legacy_function_syntax;
mod legacy_module_syntax;

pub use declaration_budget::DeclarationBudget;
pub use declaration_error::{DeclarationError, DeclarationErrorKind, DeclarationResource};
pub use legacy_anim_tree_syntax::LegacyAnimTreeSyntax;
pub use legacy_function_syntax::{LegacyFunctionSyntax, LegacyModuleItem};
pub use legacy_module_syntax::{LegacyModuleSyntax, LegacyModuleUsage};
