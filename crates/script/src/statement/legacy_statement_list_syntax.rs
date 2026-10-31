// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::statement::statement_list_cursor::StatementListCursor;
use crate::statement::statement_list_ledger::StatementListLedger;
use crate::statement::{StatementListBudget, StatementListError, StatementListErrorKind};
use crate::syntax::LegacySyntaxNode;

/// Successful complete body-list accounting, excluding allocator overhead/RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyStatementListUsage {
    /// Immediate statements/labels retained in original source order.
    pub statements: u64,
    /// Actual immutable spine visits across preflight and collection.
    pub steps: u64,
    /// Logical retained scoped-node bytes, excluding source/CST/spare capacity.
    pub metadata_bytes: u64,
}

/// Flat borrowed immediate body-list index; nested bodies are not flattened.
pub struct LegacyStatementListSyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    statements: Vec<LegacySyntaxNode<'tree, 'source>>,
    usage: LegacyStatementListUsage,
}

impl<'tree, 'source> LegacyStatementListSyntax<'tree, 'source> {
    /// Indexes an empty/nonempty/prefix body-list production in source order.
    ///
    /// Both spine passes, complete count, address arithmetic, and logical index
    /// bytes are admitted before fallible allocation. Collection/drop is flat;
    /// insertion/reversal work is bounded by the admitted immediate entry count.
    /// No nested body traversal, statement validation, binding, or execution occurs.
    ///
    /// # Errors
    /// Rejects unrelated nodes, mismatched profiles, count/storage/address overflow,
    /// exhausted budgets, and allocation failure. Actual failed work and admitted
    /// counts/storage remain visible without publishing partial indices or changing input.
    pub fn lower(
        syntax: LegacySyntaxNode<'tree, 'source>,
        budget: StatementListBudget,
    ) -> Result<Self, StatementListError> {
        let mut ledger = StatementListLedger::new(budget);
        Self::build(syntax, &mut ledger).map_err(|kind| StatementListError {
            kind,
            offset: syntax.span().start,
            work_steps: ledger.usage.steps,
            admitted_statements: ledger.usage.statements,
            admitted_metadata_bytes: ledger.usage.metadata_bytes,
        })
    }

    fn build(
        syntax: LegacySyntaxNode<'tree, 'source>,
        ledger: &mut StatementListLedger,
    ) -> Result<Self, StatementListErrorKind> {
        let mut cursor = StatementListCursor::new(syntax)?;
        while cursor.pending() {
            ledger.visit()?;
            if cursor.advance()?.is_some() {
                ledger.admit_statement()?;
            }
        }
        ledger.admit_second_pass()?;
        let count = usize::try_from(ledger.usage.statements)
            .map_err(|_| StatementListErrorKind::SizeOverflow)?;
        let mut statements = Vec::new();
        statements
            .try_reserve_exact(count)
            .map_err(|_| StatementListErrorKind::AllocationFailed)?;
        let mut cursor = StatementListCursor::new(syntax)?;
        while cursor.pending() {
            ledger.visit()?;
            if let Some(statement) = cursor.advance()? {
                statements.push(statement);
            }
        }
        statements.reverse();
        Ok(Self {
            syntax,
            statements,
            usage: ledger.usage,
        })
    }

    /// Iterates immediate statements/labels in original source order, without validating them.
    #[must_use]
    pub fn statements(
        &self,
    ) -> impl ExactSizeIterator<Item = LegacySyntaxNode<'tree, 'source>> + DoubleEndedIterator + '_
    {
        self.statements.iter().copied()
    }

    /// Returns the originally supplied scoped body-list node.
    #[must_use]
    pub fn syntax(&self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the supplied body's original half-open source byte span.
    #[must_use]
    pub fn span(&self) -> Range<u64> {
        self.syntax.span()
    }

    /// Returns successful complete logical count/work/storage accounting.
    #[must_use]
    pub fn usage(&self) -> LegacyStatementListUsage {
        self.usage
    }
}

impl fmt::Debug for LegacyStatementListSyntax<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyStatementListSyntax")
            .field("span", &self.span())
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
