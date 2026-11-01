// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_tasks::{ScopeFlow, ScopePlanner, ScopeTask, ScopeTaskKind};
use super::{FunctionScopePlan, ScopeId, invalid};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::declaration::LegacyFunctionSyntax;
use crate::statement::LegacyStatementListSyntax;
use crate::syntax::LegacySyntaxNode;

pub(in crate::compilation) fn build<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    function: &LegacyFunctionSyntax<'tree, 'source>,
) -> Result<FunctionScopePlan, FunctionCompileError> {
    let mut planner = ScopePlanner {
        plan: FunctionScopePlan::empty(),
        tasks: Vec::new(),
        contexts: Vec::new(),
        active_task: false,
    };
    let root = planner
        .plan
        .root(builder, function.parameters().len(), function.span())?;
    if root != ScopeId(0) {
        return Err(invalid(function.span()));
    }
    planner.queue_body(builder, function.body(), root, ScopeFlow::default())?;
    while let Some(ScopeTask { kind, span }) = planner.tasks.pop() {
        planner.active_task = true;
        builder.ledger.charge(1, span.clone())?;
        match kind {
            ScopeTaskKind::DeveloperEnd { parent, body } => {
                planner
                    .plan
                    .make_public(builder, parent, &[body], span.clone())?;
            }
            ScopeTaskKind::Visit {
                syntax,
                block,
                flow,
            } => planner.visit(builder, syntax, block, flow)?,
            ScopeTaskKind::IfAfterConsequence {
                parent,
                consequence,
                alternative,
                flow,
            } => {
                planner.after_consequence(
                    builder,
                    parent,
                    consequence,
                    alternative,
                    flow,
                    span.clone(),
                )?;
            }
            ScopeTaskKind::IfEnd {
                parent,
                consequence,
                alternative,
            } => {
                planner.finish_if(builder, parent, consequence, alternative, span.clone())?;
            }
            ScopeTaskKind::WhileEnd {
                parent,
                body,
                context,
            } => {
                planner.finish_while(builder, parent, body, context, span.clone())?;
            }
            ScopeTaskKind::ForAfterInitializer { parent, source } => {
                planner.after_initializer(builder, parent, source, span.clone())?;
            }
            ScopeTaskKind::ForAfterBody { work, update } => {
                planner.after_for_body(builder, work, update, span.clone())?;
            }
            ScopeTaskKind::ForEnd(work) => {
                planner.finish_for(builder, work, span.clone())?;
            }
            ScopeTaskKind::SwitchStatement {
                syntax,
                context,
                flow,
            } => {
                planner.switch_statement(builder, syntax, context, flow)?;
            }
            ScopeTaskKind::SwitchAfterStatement { context } => {
                planner.after_switch_statement(builder, context, span.clone())?;
            }
            ScopeTaskKind::SwitchEnd { parent, context } => {
                planner.finish_switch(builder, parent, context, span.clone())?;
            }
        }
        builder
            .ledger
            .release_control(size_of::<ScopeTask<'_, '_>>() as u64, span)?;
        planner.active_task = false;
    }
    if !planner.contexts.is_empty() {
        return Err(invalid(function.span()));
    }
    Ok(planner.plan)
}

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn queue_body(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        syntax: LegacySyntaxNode<'tree, 'source>,
        block: ScopeId,
        flow: ScopeFlow,
    ) -> Result<(), FunctionCompileError> {
        let body = LegacyStatementListSyntax::lower(syntax, builder.ledger.budget.body).map_err(
            |error| {
                function_compile_error::at(syntax.span(), FunctionCompileErrorKind::BodyList(error))
            },
        )?;
        for syntax in body.statements().rev() {
            builder.ledger.charge(1, syntax.span())?;
            self.push(
                builder,
                ScopeTaskKind::Visit {
                    syntax,
                    block,
                    flow,
                },
                syntax.span(),
            )?;
        }
        Ok(())
    }

    pub(super) fn visit_task(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        syntax: LegacySyntaxNode<'tree, 'source>,
        block: ScopeId,
        flow: ScopeFlow,
    ) -> Result<(), FunctionCompileError> {
        self.push(
            builder,
            ScopeTaskKind::Visit {
                syntax,
                block,
                flow,
            },
            syntax.span(),
        )
    }

    pub(super) fn work_frames(&self, span: Range<u64>) -> Result<usize, FunctionCompileError> {
        self.tasks
            .len()
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| super::overflow(span))
    }
}
