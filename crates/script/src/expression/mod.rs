// SPDX-License-Identifier: AGPL-3.0-or-later
//! Constant-cost typed operator and primary views over immutable legacy syntax.
//!
//! Descendants remain unresolved. No binding, literal validation, compilation,
//! or evaluation is implied; consumers must bound any subsequent descendant walk.

mod expression_list_budget;
mod expression_list_cursor;
mod expression_list_error;
mod expression_list_ledger;
mod legacy_call_kind;
mod legacy_call_syntax;
mod legacy_expression_error;
mod legacy_expression_kind;
mod legacy_expression_list_syntax;
mod legacy_expression_syntax;
mod legacy_function_name_syntax;
mod legacy_operators;
mod legacy_primary_kind;
mod legacy_primary_syntax;
mod legacy_syntax_profile;

pub use expression_list_budget::ExpressionListBudget;
pub use expression_list_error::{
    ExpressionListError, ExpressionListErrorKind, ExpressionListResource,
};
pub use legacy_call_kind::{LegacyCalleeKind, LegacyInvocationKind};
pub use legacy_call_syntax::LegacyCallSyntax;
pub use legacy_expression_error::LegacyExpressionError;
pub use legacy_expression_kind::LegacyExpressionKind;
pub use legacy_expression_list_syntax::{LegacyExpressionListSyntax, LegacyExpressionListUsage};
pub use legacy_expression_syntax::LegacyExpressionSyntax;
pub use legacy_function_name_syntax::LegacyFunctionNameSyntax;
pub use legacy_operators::{LegacyInfixOperator, LegacyLogicalOperator, LegacyPrefixOperator};
pub use legacy_primary_kind::{LegacyLiteralPolarity, LegacyObjectRoot, LegacyPrimaryKind};
pub use legacy_primary_syntax::LegacyPrimarySyntax;
