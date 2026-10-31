// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded structural legacy syntax trees, without original semantic actions or VM.

mod grammar;
mod legacy_actions;
mod legacy_parser;
mod legacy_productions;
mod legacy_syntax_budget;
mod legacy_syntax_error;
mod legacy_syntax_mode;
mod legacy_syntax_tree;
mod syntax_arena;
mod syntax_ledger;
mod syntax_lookahead;
mod syntax_node;
mod syntax_stack;

pub use legacy_syntax_budget::LegacySyntaxBudget;
pub use legacy_syntax_error::{LegacySyntaxError, LegacySyntaxErrorKind, LegacySyntaxResource};
pub use legacy_syntax_mode::LegacySyntaxMode;
pub use legacy_syntax_tree::{LegacySyntaxTree, LegacySyntaxUsage};
pub use syntax_node::{LegacyProductionCode, LegacySyntaxNode, LegacySyntaxNodeKind};
