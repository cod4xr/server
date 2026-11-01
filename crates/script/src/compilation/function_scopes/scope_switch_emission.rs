// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_emission::with_plan;
use super::{ScopeControl, ScopeId, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;

pub(in crate::compilation) fn enter_switch_label(
    builder: &mut FunctionBuilder<'_, '_>,
    parent: Option<ScopeId>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let parent = parent.ok_or_else(|| invalid(span.clone()))?;
        if plan.current != parent {
            if plan.block(plan.current, span.clone())?.parent != Some(parent) {
                return Err(invalid(span));
            }
            if builder.reachable {
                plan.clear_current(builder, span.clone())?;
            }
            plan.current = parent;
        }
        builder.ledger.charge(1, span.clone())?;
        let child = *plan
            .labels
            .get(&span.start)
            .ok_or_else(|| invalid(span.clone()))?;
        plan.enter(builder, parent, child, span)
    })
}

pub(in crate::compilation) fn leave_switch(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::Switch {
            parent,
            declarations_after,
            has_default,
            join,
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span));
        };
        if declarations_after > plan.block(parent, span.clone())?.names.len() {
            return Err(invalid(span));
        }
        if plan.current != parent {
            if plan.block(plan.current, span.clone())?.parent != Some(parent) {
                return Err(invalid(span));
            }
            if builder.reachable {
                plan.clear_current(builder, span.clone())?;
            }
        }
        if let Some(join) = join {
            if !has_default {
                return Err(invalid(span));
            }
            let public = plan.block(join, span.clone())?.public;
            let parent = plan.block_mut(parent, span)?;
            parent.created = parent.created.max(public);
        }
        plan.current = parent;
        Ok(())
    })
}
