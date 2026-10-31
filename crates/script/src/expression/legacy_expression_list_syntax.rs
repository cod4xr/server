// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::expression::expression_list_cursor::ExpressionListCursor;
use crate::expression::expression_list_ledger::ExpressionListLedger;
use crate::expression::{ExpressionListBudget, ExpressionListError, ExpressionListErrorKind};
use crate::syntax::LegacySyntaxNode;

/// Successful complete list accounting; not allocator overhead or aggregate VM use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyExpressionListUsage {
    /// Retained expressions in original source order.
    pub expressions: u64,
    /// Actual node visits across preflight and collection.
    pub steps: u64,
    /// Logical retained node-index bytes, excluding input source/CST and spare capacity.
    pub metadata_bytes: u64,
}

/// Flat source-ordered index borrowing immutable expression nodes from one tree.
///
/// Source order is not a claim about native argument evaluation/push order. No
/// literal values, group/vector/call arity, binding, or executable capacity is
/// validated. Dropping the index does not recursively drop expression descendants.
pub struct LegacyExpressionListSyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    expressions: Vec<LegacySyntaxNode<'tree, 'source>>,
    usage: LegacyExpressionListUsage,
}

impl<'tree, 'source> LegacyExpressionListSyntax<'tree, 'source> {
    /// Lowers an optional, empty, or nonempty/prefix expression-list production.
    ///
    /// Both immutable spine passes, complete retained count, and logical metadata
    /// are admitted before fallible output allocation. No recursion, source copy,
    /// fixed native argument ceiling, descendant traversal, or evaluation occurs.
    /// Vector insertion/reversal work is bounded by the admitted entry count.
    ///
    /// # Errors
    /// Rejects unrelated nodes, profile mismatches, count/address-space overflow,
    /// exhausted budgets, and allocation failures. Errors identify the supplied
    /// list's original start. No partial index escapes or input state changes.
    pub fn lower(
        syntax: LegacySyntaxNode<'tree, 'source>,
        budget: ExpressionListBudget,
    ) -> Result<Self, ExpressionListError> {
        Self::build(syntax, budget).map_err(|kind| ExpressionListError {
            kind,
            offset: syntax.span().start,
        })
    }

    fn build(
        syntax: LegacySyntaxNode<'tree, 'source>,
        budget: ExpressionListBudget,
    ) -> Result<Self, ExpressionListErrorKind> {
        let mut ledger = ExpressionListLedger::new(budget);
        let mut cursor = ExpressionListCursor::new(syntax);
        while cursor.pending() {
            ledger.visit()?;
            if cursor.advance()?.is_some() {
                ledger.admit_expression()?;
            }
        }
        ledger.admit_second_pass()?;
        let count = usize::try_from(ledger.usage.expressions)
            .map_err(|_| ExpressionListErrorKind::SizeOverflow)?;
        let mut expressions = Vec::new();
        expressions
            .try_reserve_exact(count)
            .map_err(|_| ExpressionListErrorKind::AllocationFailed)?;
        let mut cursor = ExpressionListCursor::new(syntax);
        while cursor.pending() {
            if let Some(expression) = cursor.advance()? {
                expressions.push(expression);
            }
        }
        expressions.reverse();
        Ok(Self {
            syntax,
            expressions,
            usage: ledger.usage,
        })
    }

    /// Iterates original source order; reverse iteration makes no runtime-order claim.
    #[must_use]
    pub fn expressions(
        &self,
    ) -> impl ExactSizeIterator<Item = LegacySyntaxNode<'tree, 'source>> + DoubleEndedIterator + '_
    {
        self.expressions.iter().copied()
    }

    /// Returns the original supplied list node without replacing its scoped identity.
    #[must_use]
    pub fn syntax(&self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the supplied list node's original half-open byte span.
    #[must_use]
    pub fn span(&self) -> Range<u64> {
        self.syntax.span()
    }

    /// Returns successful logical storage/work accounting, without source contents.
    #[must_use]
    pub fn usage(&self) -> LegacyExpressionListUsage {
        self.usage
    }
}

impl fmt::Debug for LegacyExpressionListSyntax<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyExpressionListSyntax")
            .field("span", &self.span())
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
