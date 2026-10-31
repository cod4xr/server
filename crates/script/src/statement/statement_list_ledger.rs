// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::statement::statement_list_error;
use crate::statement::{
    LegacyStatementListUsage, StatementListBudget, StatementListErrorKind, StatementListResource,
};
use crate::syntax::LegacySyntaxNode;

pub(super) struct StatementListLedger {
    pub(super) budget: StatementListBudget,
    pub(super) usage: LegacyStatementListUsage,
}

impl StatementListLedger {
    pub(super) fn new(budget: StatementListBudget) -> Self {
        Self {
            budget,
            usage: LegacyStatementListUsage {
                statements: 0,
                steps: 0,
                metadata_bytes: 0,
            },
        }
    }

    pub(super) fn visit(&mut self) -> Result<(), StatementListErrorKind> {
        let next = self
            .usage
            .steps
            .checked_add(1)
            .ok_or(StatementListErrorKind::SizeOverflow)?;
        statement_list_error::check(StatementListResource::Steps, next, self.budget.max_steps)?;
        self.usage.steps = next;
        Ok(())
    }

    pub(super) fn admit_statement(&mut self) -> Result<(), StatementListErrorKind> {
        let count = self
            .usage
            .statements
            .checked_add(1)
            .ok_or(StatementListErrorKind::SizeOverflow)?;
        statement_list_error::check(
            StatementListResource::Statements,
            count,
            self.budget.max_statements,
        )?;
        let bytes = count
            .checked_mul(size_of::<LegacySyntaxNode<'_, '_>>() as u64)
            .ok_or(StatementListErrorKind::SizeOverflow)?;
        statement_list_error::check(
            StatementListResource::MetadataBytes,
            bytes,
            self.budget.max_metadata_bytes,
        )?;
        self.usage.statements = count;
        self.usage.metadata_bytes = bytes;
        Ok(())
    }

    pub(super) fn admit_second_pass(&self) -> Result<(), StatementListErrorKind> {
        let total = self
            .usage
            .steps
            .checked_mul(2)
            .ok_or(StatementListErrorKind::SizeOverflow)?;
        // Preflight work is actual work; this check admits future visits without
        // charging them before allocation or collection succeeds.
        statement_list_error::check(StatementListResource::Steps, total, self.budget.max_steps)
    }
}
