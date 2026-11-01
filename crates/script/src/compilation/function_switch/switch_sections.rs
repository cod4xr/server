// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_initialization;
use crate::compilation::function_loop_transfer::FunctionBreakOwner;
use crate::compilation::function_scopes;
use crate::compilation::function_switch::switch_context::SwitchLabel;
use crate::compilation::function_switch::switch_error;

pub(super) fn label(
    builder: &mut FunctionBuilder<'_, '_>,
    label: SwitchLabel,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    if builder.break_owner != Some(FunctionBreakOwner::Switch) {
        return Err(switch_error::invalid(span));
    }
    let scope_parent = builder
        .active_switch
        .as_ref()
        .ok_or_else(|| switch_error::invalid(span.clone()))?
        .scope_parent;
    function_scopes::enter_switch_label(builder, scope_parent, span.clone())?;
    builder.break_scope = function_scopes::current(builder);
    let context = builder
        .active_switch
        .as_mut()
        .ok_or_else(|| switch_error::invalid(span.clone()))?;
    let incoming = context
        .incoming
        .take()
        .ok_or_else(|| switch_error::invalid(span.clone()))?;
    function_initialization::merge_from(builder, &incoming, span.clone())?;
    let target = builder.instructions.len();
    let context = builder
        .active_switch
        .as_mut()
        .ok_or_else(|| switch_error::invalid(span.clone()))?;
    context.incoming = Some(incoming);
    let slot = match label {
        SwitchLabel::Case(index) => {
            &mut context
                .cases
                .get_mut(index)
                .ok_or_else(|| switch_error::invalid(span.clone()))?
                .target
        }
        SwitchLabel::Default => &mut context.default_target,
    };
    if slot.replace(target).is_some() {
        return Err(switch_error::invalid(span));
    }
    Ok(())
}

pub(in crate::compilation) fn record_break(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    if builder.break_owner != Some(FunctionBreakOwner::Switch) {
        return Err(switch_error::invalid(span));
    }
    function_scopes::break_edge(builder, builder.break_scope, span.clone())?;
    let context = builder
        .active_switch
        .as_mut()
        .ok_or_else(|| switch_error::invalid(span.clone()))?;
    let previous = context.exit_facts.take();
    let chain = context.breaks.take();
    if let Some(previous) = previous {
        function_initialization::merge(builder, previous, span.clone())?;
    }
    let facts = function_initialization::capture(builder, span.clone())?;
    let site = builder.transfer_jump(chain, span.clone())?;
    let context = builder
        .active_switch
        .as_mut()
        .ok_or_else(|| switch_error::invalid(span.clone()))?;
    context.exit_facts = Some(facts);
    context.breaks = Some(site);
    builder.reachable = false;
    Ok(())
}
