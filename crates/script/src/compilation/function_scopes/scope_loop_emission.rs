// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_emission::with_plan;
use super::{ScopeControl, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;

pub(in crate::compilation) fn prepare_loop(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::Loop {
            parent,
            body,
            declarations_after,
            ..
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span));
        };
        if plan.current != parent
            || declarations_after > plan.block(parent, span.clone())?.names.len()
        {
            return Err(invalid(span));
        }
        // Transfer first: inherited names can expand the body's old public
        // prefix. A fixed max(old_count, parent_created) loses displaced names.
        plan.transfer(builder, parent, body, span.clone())?;
        let public = plan.block(body, span.clone())?.public;
        let parent = plan.block_mut(parent, span)?;
        parent.created = parent.created.max(public);
        Ok(())
    })
}

pub(in crate::compilation) fn enter_loop_body(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::Loop { parent, body, .. } = plan.control(builder, span.clone())? else {
            return Err(invalid(span));
        };
        plan.enter(builder, parent, body, span)
    })
}

pub(in crate::compilation) fn enter_for_update(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::Loop {
            parent,
            body,
            update: Some(update),
            ..
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span));
        };
        if plan.current != body {
            return Err(invalid(span));
        }
        plan.current = parent;
        plan.enter(builder, parent, update, span)
    })
}

pub(in crate::compilation) fn leave_loop(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<bool, FunctionCompileError> {
    let Some(mut plan) = builder.scopes.take() else {
        return Ok(true);
    };
    let result = (|| {
        let ScopeControl::Loop {
            parent,
            body,
            update,
            exits,
            ..
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span.clone()));
        };
        if plan.current != update.unwrap_or(body) {
            return Err(invalid(span.clone()));
        }
        plan.current = parent;
        Ok(exits)
    })();
    builder.scopes = Some(plan);
    result
}
