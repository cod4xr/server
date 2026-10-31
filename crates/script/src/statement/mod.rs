// SPDX-License-Identifier: AGPL-3.0-or-later
//! Borrowed local statement roles and separately admitted ordered body lists.
//!
//! Binding, lvalue/control-flow legality, developer activation, scheduling,
//! event argument semantics, and execution remain separate responsibilities.

mod legacy_statement_kind;
mod legacy_statement_list_syntax;
mod legacy_statement_syntax;
mod statement_error;
mod statement_list_budget;
mod statement_list_cursor;
mod statement_list_error;
mod statement_list_ledger;
mod statement_profile;

pub use legacy_statement_kind::{
    LegacyEventKind, LegacyProfileKind, LegacyStatementKind, LegacyUpdateKind,
};
pub use legacy_statement_list_syntax::{LegacyStatementListSyntax, LegacyStatementListUsage};
pub use legacy_statement_syntax::LegacyStatementSyntax;
pub use statement_error::LegacyStatementError;
pub use statement_list_budget::StatementListBudget;
pub use statement_list_error::{StatementListError, StatementListErrorKind, StatementListResource};
