// SPDX-License-Identifier: AGPL-3.0-or-later
//! Constant-cost typed operator and primary views over immutable legacy syntax.
//!
//! Descendants remain unresolved. No binding, literal validation, compilation,
//! or evaluation is implied; consumers must bound any subsequent descendant walk.

mod legacy_expression_error;
mod legacy_expression_kind;
mod legacy_expression_syntax;
mod legacy_operators;
mod legacy_primary_kind;
mod legacy_primary_syntax;
mod legacy_syntax_profile;

pub use legacy_expression_error::LegacyExpressionError;
pub use legacy_expression_kind::LegacyExpressionKind;
pub use legacy_expression_syntax::LegacyExpressionSyntax;
pub use legacy_operators::{LegacyInfixOperator, LegacyLogicalOperator, LegacyPrefixOperator};
pub use legacy_primary_kind::{LegacyLiteralPolarity, LegacyObjectRoot, LegacyPrimaryKind};
pub use legacy_primary_syntax::LegacyPrimarySyntax;
