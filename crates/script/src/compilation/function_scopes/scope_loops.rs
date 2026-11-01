// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_condition::{self, ScopeCondition};
use super::scope_tasks::{ScopeContext, ScopeFlow, ScopePlanner, ScopeTaskKind};
use super::{ScopeAbort, ScopeControl, ScopeId, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn begin_while(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        condition: crate::syntax::LegacySyntaxNode<'tree, 'source>,
        body: crate::syntax::LegacySyntaxNode<'tree, 'source>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let condition =
            scope_condition::classify(builder, condition, self.work_frames(span.clone())?)?;
        let context = self.context(builder, parent, condition, false, span.clone())?;
        let child = self.plan.clone_block(builder, parent, span.clone())?;
        self.plan.insert_control(
            builder,
            ScopeControl::Loop {
                parent,
                body: child,
                update: None,
                declarations_after: 0,
                exits: true,
            },
            span.clone(),
        )?;
        self.push(
            builder,
            ScopeTaskKind::WhileEnd {
                parent,
                body: child,
                context,
            },
            span,
        )?;
        self.visit_task(
            builder,
            body,
            child,
            ScopeFlow {
                continuation: Some(context),
                breaking: Some(context),
            },
        )
    }

    pub(super) fn finish_while(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        body: ScopeId,
        context: usize,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        self.implicit_continue(builder, context, body, span.clone())?;
        let context = self.pop_context(context, span.clone())?;
        if context.parent != parent || context.is_switch {
            return Err(invalid(span));
        }
        self.plan
            .union_names(builder, parent, &context.continues, span.clone())?;
        self.finish_loop(builder, parent, &[body], &context, span.clone())?;
        Self::release_context(context, builder, span)
    }

    pub(super) fn finish_loop(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        blocks: &[ScopeId],
        context: &ScopeContext,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let constant_true = matches!(context.condition, ScopeCondition::Always(true));
        if constant_true {
            for block in &context.breaks {
                builder.ledger.charge(1, span.clone())?;
                self.plan.block_mut(*block, span.clone())?.abort = ScopeAbort::Open;
            }
            self.plan
                .intersect_names(builder, parent, &context.breaks, span.clone())?;
        }
        let exits = !constant_true || !context.breaks.is_empty();
        self.plan.block_mut(parent, span.clone())?.abort = if exits {
            ScopeAbort::Open
        } else {
            ScopeAbort::Return
        };
        let count = self
            .plan
            .make_public(builder, parent, blocks, span.clone())?;
        let Some(ScopeControl::Loop {
            declarations_after,
            exits: slot,
            ..
        }) = self.plan.controls.get_mut(&span.start)
        else {
            return Err(invalid(span));
        };
        *declarations_after = count;
        *slot = exits;
        Ok(())
    }
}
