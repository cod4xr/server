// SPDX-License-Identifier: AGPL-3.0-or-later
//! Constant-cost typed operator views over immutable legacy concrete syntax.
//!
//! Primary forms remain opaque. No binding, literal validation, compilation, or
//! evaluation is implied; consumers must bound any subsequent descendant walk.

mod legacy_expression_error;
mod legacy_expression_kind;
mod legacy_expression_syntax;
mod legacy_operators;

pub use legacy_expression_error::LegacyExpressionError;
pub use legacy_expression_kind::LegacyExpressionKind;
pub use legacy_expression_syntax::LegacyExpressionSyntax;
pub use legacy_operators::{LegacyInfixOperator, LegacyLogicalOperator, LegacyPrefixOperator};
