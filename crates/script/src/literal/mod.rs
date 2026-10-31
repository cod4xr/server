// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded legacy literal values and owned plain-string concatenation.
//!
//! No interning, semantic AST ownership, aggregate heap accounting, or VM execution.

mod legacy_literal;
mod legacy_literal_budget;
mod legacy_literal_error;
mod legacy_string;
mod legacy_string_error;
mod number_literal;

pub use legacy_literal::LegacyLiteral;
pub use legacy_literal_budget::LegacyLiteralBudget;
pub use legacy_literal_error::{LegacyLiteralError, LegacyLiteralResource};
pub use legacy_string::{LegacyString, LegacyStringKind};
pub use legacy_string_error::LegacyStringError;
