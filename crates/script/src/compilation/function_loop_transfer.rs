// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_for_update;
use crate::compilation::function_initialization::FunctionInitializationSnapshot;
use crate::compilation::function_scopes::{self, ScopeId};
use crate::compilation::function_switch;
use crate::compilation::function_task::{FunctionLoopHead, FunctionPatchSite};
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;

pub(super) struct FunctionLoopContext {
    previous_break_owner: Option<FunctionBreakOwner>,
    previous_break_scope: Option<ScopeId>,
    pub(super) head: FunctionLoopHead,
    pub(super) breaks: Option<FunctionPatchSite>,
    pub(super) continuation: FunctionContinueDestination,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum FunctionBreakOwner {
    Loop,
    Switch,
}

pub(super) enum FunctionContinueDestination {
    Condition,
    Update {
        sites: Option<FunctionPatchSite>,
        facts: Option<FunctionInitializationSnapshot>,
    },
}

#[derive(Clone, Copy)]
pub(super) enum FunctionLoopTransfer {
    Break,
    Continue,
}

pub(super) fn enter(
    builder: &mut FunctionBuilder<'_, '_>,
    head: FunctionLoopHead,
    continuation: FunctionContinueDestination,
) -> Option<FunctionLoopContext> {
    let scope = function_scopes::current(builder);
    let previous_break_scope = std::mem::replace(&mut builder.break_scope, scope);
    builder.active_loop.replace(FunctionLoopContext {
        previous_break_owner: builder.break_owner.replace(FunctionBreakOwner::Loop),
        previous_break_scope,
        head,
        breaks: None,
        continuation,
    })
}

pub(super) fn leave(
    builder: &mut FunctionBuilder<'_, '_>,
    head: FunctionLoopHead,
    outer_loop: Option<FunctionLoopContext>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let context = builder
        .active_loop
        .take()
        .ok_or_else(|| invalid(span.clone()))?;
    if context.head != head || builder.break_owner != Some(FunctionBreakOwner::Loop) {
        return Err(invalid(span));
    }
    if matches!(
        context.continuation,
        FunctionContinueDestination::Update { sites: Some(_), .. }
            | FunctionContinueDestination::Update { facts: Some(_), .. }
    ) {
        return Err(invalid(span));
    }
    builder.bind_transfers(context.breaks, span)?;
    builder.break_owner = context.previous_break_owner;
    builder.break_scope = context.previous_break_scope;
    builder.active_loop = outer_loop;
    Ok(())
}

pub(super) fn lower(
    builder: &mut FunctionBuilder<'_, '_>,
    kind: FunctionLoopTransfer,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    if matches!(kind, FunctionLoopTransfer::Break)
        && builder.break_owner == Some(FunctionBreakOwner::Switch)
    {
        return function_switch::record_break(builder, span);
    }
    let Some(context) = builder.active_loop.as_ref() else {
        let kind = match kind {
            FunctionLoopTransfer::Break => FunctionCompileErrorKind::IllegalBreak,
            FunctionLoopTransfer::Continue => FunctionCompileErrorKind::IllegalContinue,
        };
        return Err(function_compile_error::at(span, kind));
    };
    let head = context.head;
    let condition_destination =
        matches!(context.continuation, FunctionContinueDestination::Condition);
    match kind {
        FunctionLoopTransfer::Continue => {
            function_scopes::continue_edge(builder, span.clone())?;
            if condition_destination {
                builder.emit(KernelInstruction::Jump(head.0), span)?;
            } else {
                function_for_update::record_continue(builder, span)?;
            }
        }
        FunctionLoopTransfer::Break => {
            function_scopes::break_edge(builder, builder.break_scope, span.clone())?;
            let chain = builder
                .active_loop
                .as_mut()
                .ok_or_else(|| invalid(span.clone()))?
                .breaks
                .take();
            let site = builder.transfer_jump(chain, span.clone())?;
            builder
                .active_loop
                .as_mut()
                .ok_or_else(|| invalid(span))?
                .breaks = Some(site);
        }
    }
    builder.reachable = false;
    Ok(())
}

fn invalid(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::InvalidLoopContext)
}
