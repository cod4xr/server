// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-grounded legacy lexemes without literal decoding, interning, or a parser.

mod legacy_lex_budget;
mod legacy_lex_error;
mod legacy_lex_usage;
mod legacy_lexeme;
mod legacy_lexer;
mod legacy_scanner;
mod legacy_tables;

pub use legacy_lex_budget::LegacyLexBudget;
pub use legacy_lex_error::{LegacyLexError, LegacyLexErrorKind, LegacyLexResource};
pub use legacy_lex_usage::LegacyLexUsage;
pub use legacy_lexeme::{LegacyLexeme, LegacyLexemeKind, LegacyTokenCode};
pub use legacy_lexer::LegacyLexer;
