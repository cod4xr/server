// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;

use crate::expression::expression_list_error;
use crate::expression::{
    ExpressionListBudget, ExpressionListErrorKind, ExpressionListResource,
    LegacyExpressionListUsage,
};
use crate::syntax::LegacySyntaxNode;

pub(super) struct ExpressionListLedger {
    pub(super) budget: ExpressionListBudget,
    pub(super) usage: LegacyExpressionListUsage,
}

impl ExpressionListLedger {
    pub(super) fn new(budget: ExpressionListBudget) -> Self {
        Self {
            budget,
            usage: LegacyExpressionListUsage {
                expressions: 0,
                steps: 0,
                metadata_bytes: 0,
            },
        }
    }

    pub(super) fn visit(&mut self) -> Result<(), ExpressionListErrorKind> {
        let next = self
            .usage
            .steps
            .checked_add(1)
            .ok_or(ExpressionListErrorKind::SizeOverflow)?;
        expression_list_error::check(ExpressionListResource::Steps, next, self.budget.max_steps)?;
        self.usage.steps = next;
        Ok(())
    }

    pub(super) fn admit_expression(&mut self) -> Result<(), ExpressionListErrorKind> {
        let count = self
            .usage
            .expressions
            .checked_add(1)
            .ok_or(ExpressionListErrorKind::SizeOverflow)?;
        expression_list_error::check(
            ExpressionListResource::Expressions,
            count,
            self.budget.max_expressions,
        )?;
        let bytes = count
            .checked_mul(mem::size_of::<LegacySyntaxNode<'_, '_>>() as u64)
            .ok_or(ExpressionListErrorKind::SizeOverflow)?;
        expression_list_error::check(
            ExpressionListResource::MetadataBytes,
            bytes,
            self.budget.max_metadata_bytes,
        )?;
        self.usage.expressions = count;
        self.usage.metadata_bytes = bytes;
        Ok(())
    }

    pub(super) fn admit_second_pass(&mut self) -> Result<(), ExpressionListErrorKind> {
        let total = self
            .usage
            .steps
            .checked_mul(2)
            .ok_or(ExpressionListErrorKind::SizeOverflow)?;
        expression_list_error::check(ExpressionListResource::Steps, total, self.budget.max_steps)?;
        self.usage.steps = total;
        Ok(())
    }
}
