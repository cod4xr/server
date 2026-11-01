// SPDX-License-Identifier: AGPL-3.0-or-later
use super::scope_emission::with_plan;
use super::scope_tasks::{ScopeFlow, ScopePlanner, ScopeTaskKind};
use super::{ScopeControl, ScopeId, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::syntax::LegacySyntaxNode;
use std::ops::Range;

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn begin_developer(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        syntax: LegacySyntaxNode<'tree, 'source>,
        flow: ScopeFlow,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let body = self.plan.clone_block(builder, parent, span.clone())?;
        self.plan.insert_control(
            builder,
            ScopeControl::Developer { parent, body },
            span.clone(),
        )?;
        self.push(builder, ScopeTaskKind::DeveloperEnd { parent, body }, span)?;
        self.queue_body(builder, syntax, body, flow)
    }
}

pub(in crate::compilation) fn enter_developer(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::Developer { parent, body } = plan.control(builder, span.clone())? else {
            return Err(invalid(span));
        };
        plan.enter(builder, parent, body, span)
    })
}

pub(in crate::compilation) fn leave_developer(
    builder: &mut FunctionBuilder<'_, '_>,
    discard: bool,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::Developer { parent, body } = plan.control(builder, span.clone())? else {
            return Err(invalid(span));
        };
        if plan.current != body {
            return Err(invalid(span));
        }
        if builder.reachable && !discard {
            plan.clear_current(builder, span)?;
        }
        // The developer child's created/initialized/abort state never publishes
        // directly into the parent, even for an enabled returning body.
        plan.current = parent;
        Ok(())
    })
}
