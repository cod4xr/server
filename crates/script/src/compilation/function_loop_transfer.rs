// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_for_update;
use crate::compilation::function_initialization::FunctionInitializationSnapshot;
use crate::compilation::function_task::{FunctionLoopHead, FunctionPatchSite};
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;

pub(super) struct FunctionLoopContext {
    pub(super) head: FunctionLoopHead,
    pub(super) breaks: Option<FunctionPatchSite>,
    pub(super) continuation: FunctionContinueDestination,
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
    builder.active_loop.replace(FunctionLoopContext {
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
    if context.head != head {
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
    builder.active_loop = outer_loop;
    Ok(())
}

pub(super) fn lower(
    builder: &mut FunctionBuilder<'_, '_>,
    kind: FunctionLoopTransfer,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let Some(context) = builder.active_loop.as_ref() else {
        let kind = match kind {
            FunctionLoopTransfer::Break => FunctionCompileErrorKind::IllegalBreak,
            FunctionLoopTransfer::Continue => FunctionCompileErrorKind::IllegalContinue,
        };
        return Err(function_compile_error::at(span, kind));
    };
    match kind {
        FunctionLoopTransfer::Continue => match &context.continuation {
            FunctionContinueDestination::Condition => {
                builder.emit(KernelInstruction::Jump(context.head.0), span)?;
            }
            FunctionContinueDestination::Update { .. } => {
                function_for_update::record_continue(builder, span)?;
            }
        },
        FunctionLoopTransfer::Break => {
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
