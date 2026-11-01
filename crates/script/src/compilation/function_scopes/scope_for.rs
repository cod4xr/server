// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_condition::{self, ScopeCondition};
use super::scope_tasks::{ScopeFlow, ScopeForSource, ScopeForWork, ScopePlanner, ScopeTaskKind};
use super::{ScopeControl, ScopeId, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::syntax::LegacySyntaxNode;

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn after_initializer(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        source: ScopeForSource<'tree, 'source>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let condition = if let Some(condition) = source.condition {
            scope_condition::classify(builder, condition, self.work_frames(span.clone())?)?
        } else {
            ScopeCondition::Always(true)
        };
        let context = self.context(builder, parent, condition, false, span.clone())?;
        let body = self.plan.clone_block(builder, parent, span.clone())?;
        let update = self.plan.clone_block(builder, parent, span.clone())?;
        self.plan.insert_control(
            builder,
            ScopeControl::Loop {
                parent,
                body,
                update: Some(update),
                declarations_after: 0,
                exits: true,
            },
            span.clone(),
        )?;
        self.push(
            builder,
            ScopeTaskKind::ForAfterBody {
                work: ScopeForWork {
                    parent,
                    body,
                    update,
                    context,
                },
                update: source.update,
            },
            span,
        )?;
        self.visit_task(
            builder,
            source.body,
            body,
            ScopeFlow {
                continuation: Some(context),
                breaking: Some(context),
            },
        )
    }

    pub(super) fn after_for_body(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        work: ScopeForWork,
        update: Option<LegacySyntaxNode<'tree, 'source>>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        self.implicit_continue(builder, work.context, work.body, span.clone())?;
        let rows = &self
            .contexts
            .get(work.context)
            .ok_or_else(|| invalid(span.clone()))?
            .continues;
        self.plan
            .union_names(builder, work.update, rows, span.clone())?;
        self.push(builder, ScopeTaskKind::ForEnd(work), span)?;
        if let Some(update) = update {
            self.visit_task(builder, update, work.update, ScopeFlow::default())?;
        }
        Ok(())
    }

    pub(super) fn finish_for(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        work: ScopeForWork,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let context = self.pop_context(work.context, span.clone())?;
        if context.parent != work.parent || context.is_switch {
            return Err(invalid(span));
        }
        self.plan
            .union_names(builder, work.parent, &[work.update], span.clone())?;
        self.finish_loop(
            builder,
            work.parent,
            &[work.body, work.update],
            &context,
            span.clone(),
        )?;
        Self::release_context(context, builder, span)
    }
}
