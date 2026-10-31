// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded legacy literal values without interning, AST ownership, or VM execution.

mod legacy_literal;
mod legacy_literal_budget;
mod legacy_literal_error;
mod legacy_string;
mod number_literal;

pub use legacy_literal::LegacyLiteral;
pub use legacy_literal_budget::LegacyLiteralBudget;
pub use legacy_literal_error::{LegacyLiteralError, LegacyLiteralResource};
pub use legacy_string::{LegacyString, LegacyStringKind};
