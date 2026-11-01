// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_initialization;
use crate::compilation::function_loop_transfer::FunctionBreakOwner;
use crate::compilation::function_switch::switch_context::FunctionSwitchContext;
use crate::compilation::function_switch::{switch_dispatch, switch_error};
use crate::compilation::function_task::FunctionPatchSite;

pub(super) fn finish(
    builder: &mut FunctionBuilder<'_, '_>,
    head: usize,
    dispatch: FunctionPatchSite,
    outer: Option<FunctionSwitchContext>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let mut context = builder
        .active_switch
        .take()
        .ok_or_else(|| switch_error::invalid(span.clone()))?;
    if context.head != head
        || builder.break_owner != Some(FunctionBreakOwner::Switch)
        || context.default_span.is_some() != context.default_target.is_some()
    {
        return Err(switch_error::invalid(span));
    }
    if let Some(facts) = context.exit_facts.take() {
        function_initialization::merge(builder, facts, span.clone())?;
    }
    let incoming = context
        .incoming
        .take()
        .ok_or_else(|| switch_error::invalid(span.clone()))?;
    if context.default_span.is_none() {
        function_initialization::merge(builder, incoming, span.clone())?;
    } else {
        function_initialization::release(builder, incoming, span.clone())?;
    }
    // Body fallthrough bypasses the dispatcher; matching dispatch consumes its key
    // before jumping backward to a case body. All exits share the following join.
    let fallthrough = builder.jump(span.clone())?;
    builder.bind(dispatch, span.clone())?;
    switch_dispatch::prepare(builder, &mut context.cases, span.clone())?;
    let breaks = switch_dispatch::emit(
        builder,
        &context.cases,
        context.default_target,
        context.breaks,
        span.clone(),
    )?;
    drop(context.cases);
    builder
        .ledger
        .release_control(context.case_bytes, span.clone())?;
    builder.bind_transfers(breaks, span.clone())?;
    builder.bind(fallthrough, span)?;
    builder.break_owner = context.previous_break_owner;
    builder.active_switch = outer;
    Ok(())
}
