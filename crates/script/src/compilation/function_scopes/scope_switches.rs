// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_condition::ScopeCondition;
use super::scope_tasks::{ScopeFlow, ScopePlanner, ScopeTaskKind};
use super::{ScopeAbort, ScopeControl, ScopeId, allocation, invalid};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::statement::{LegacyStatementKind, LegacyStatementListSyntax, LegacyStatementSyntax};
use crate::syntax::LegacySyntaxNode;

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn begin_switch(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        body: LegacySyntaxNode<'tree, 'source>,
        flow: ScopeFlow,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let context = self.context(builder, parent, ScopeCondition::Dynamic, true, span.clone())?;
        self.plan.insert_control(
            builder,
            ScopeControl::Switch {
                parent,
                declarations_after: 0,
                has_default: false,
                join: None,
            },
            span.clone(),
        )?;
        self.push(builder, ScopeTaskKind::SwitchEnd { parent, context }, span)?;
        let body = LegacyStatementListSyntax::lower(body, builder.ledger.budget.body).map_err(
            |error| {
                function_compile_error::at(body.span(), FunctionCompileErrorKind::BodyList(error))
            },
        )?;
        let flow = ScopeFlow {
            continuation: flow.continuation,
            breaking: Some(context),
        };
        for syntax in body.statements().rev() {
            builder.ledger.charge(1, syntax.span())?;
            self.push(
                builder,
                ScopeTaskKind::SwitchStatement {
                    syntax,
                    context,
                    flow,
                },
                syntax.span(),
            )?;
        }
        Ok(())
    }

    pub(super) fn switch_statement(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        syntax: LegacySyntaxNode<'tree, 'source>,
        context: usize,
        flow: ScopeFlow,
    ) -> Result<(), FunctionCompileError> {
        let view = LegacyStatementSyntax::from_node(syntax).map_err(|_| invalid(syntax.span()))?;
        if matches!(
            view.kind(),
            LegacyStatementKind::Case { .. } | LegacyStatementKind::Default
        ) {
            let parent = self
                .contexts
                .get(context)
                .ok_or_else(|| invalid(syntax.span()))?
                .parent;
            let block = self.plan.clone_block(builder, parent, syntax.span())?;
            self.plan
                .insert_label(builder, block, view.statement_syntax().span())?;
            let context = self
                .contexts
                .get_mut(context)
                .ok_or_else(|| invalid(syntax.span()))?;
            context.current_case = Some(block);
            context.has_default |= matches!(view.kind(), LegacyStatementKind::Default);
            return Ok(());
        }
        let Some(block) = self
            .contexts
            .get(context)
            .ok_or_else(|| invalid(syntax.span()))?
            .current_case
        else {
            // The normal switch inventory rejects pre-label/unreachable syntax.
            return Ok(());
        };
        self.push(
            builder,
            ScopeTaskKind::SwitchAfterStatement { context },
            syntax.span(),
        )?;
        self.visit_task(builder, syntax, block, flow)
    }

    pub(super) fn after_switch_statement(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        context: usize,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let block = self
            .contexts
            .get(context)
            .ok_or_else(|| invalid(span.clone()))?
            .current_case
            .ok_or_else(|| invalid(span.clone()))?;
        let abort = self.plan.block(block, span.clone())?.abort;
        if abort == ScopeAbort::Open {
            return Ok(());
        }
        let context = self
            .contexts
            .get_mut(context)
            .ok_or_else(|| invalid(span.clone()))?;
        if abort == ScopeAbort::Break {
            builder.admit_control(size_of::<ScopeId>() as u64, span.clone())?;
            context
                .public_cases
                .try_reserve(1)
                .map_err(|_| allocation(span.clone()))?;
            context.public_cases.push(block);
            self.plan.block_mut(block, span)?.abort = ScopeAbort::Open;
            context.exit_abort = ScopeAbort::Open;
        } else {
            context.exit_abort = context.exit_abort.min(abort);
        }
        context.current_case = None;
        Ok(())
    }

    pub(super) fn finish_switch(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        index: usize,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let mut context = self.pop_context(index, span.clone())?;
        if context.parent != parent || !context.is_switch {
            return Err(invalid(span));
        }
        if context.has_default {
            if let Some(last) = context.current_case {
                if self.plan.block(last, span.clone())?.abort != ScopeAbort::Open {
                    return Err(invalid(span));
                }
                builder.admit_control((2 * size_of::<ScopeId>()) as u64, span.clone())?;
                context
                    .breaks
                    .try_reserve(1)
                    .map_err(|_| allocation(span.clone()))?;
                context
                    .public_cases
                    .try_reserve(1)
                    .map_err(|_| allocation(span.clone()))?;
                context.breaks.push(last);
                context.public_cases.push(last);
                context.exit_abort = ScopeAbort::Open;
            }
            self.plan.block_mut(parent, span.clone())?.abort = context.exit_abort;
            if context.exit_abort == ScopeAbort::Open {
                for block in &context.breaks {
                    builder.ledger.charge(1, span.clone())?;
                    self.plan.block_mut(*block, span.clone())?.abort = ScopeAbort::Open;
                }
                self.plan
                    .intersect_names(builder, parent, &context.breaks, span.clone())?;
                self.plan
                    .make_public(builder, parent, &context.public_cases, span.clone())?;
            }
        }
        let count = self.plan.block(parent, span.clone())?.names.len();
        let Some(ScopeControl::Switch {
            declarations_after,
            has_default,
            join,
            ..
        }) = self.plan.controls.get_mut(&span.start)
        else {
            return Err(invalid(span));
        };
        *declarations_after = count;
        *has_default = context.has_default;
        *join = if context.has_default && context.exit_abort == ScopeAbort::Open {
            context.public_cases.first().copied()
        } else {
            None
        };
        Self::release_context(context, builder, span)
    }
}
