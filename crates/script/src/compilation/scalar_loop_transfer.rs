// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_for_update;
use crate::compilation::scalar_initialization::ScalarInitializationSnapshot;
use crate::compilation::scalar_task::{ScalarLoopHead, ScalarPatchSite};
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::KernelInstruction;

pub(super) struct ScalarLoopContext {
    pub(super) head: ScalarLoopHead,
    pub(super) breaks: Option<ScalarPatchSite>,
    pub(super) continuation: ScalarContinueDestination,
}

pub(super) enum ScalarContinueDestination {
    Condition,
    Update {
        sites: Option<ScalarPatchSite>,
        facts: Option<ScalarInitializationSnapshot>,
    },
}

#[derive(Clone, Copy)]
pub(super) enum ScalarLoopTransfer {
    Break,
    Continue,
}

pub(super) fn enter(
    builder: &mut ScalarBuilder<'_, '_>,
    head: ScalarLoopHead,
    continuation: ScalarContinueDestination,
) -> Option<ScalarLoopContext> {
    builder.active_loop.replace(ScalarLoopContext {
        head,
        breaks: None,
        continuation,
    })
}

pub(super) fn leave(
    builder: &mut ScalarBuilder<'_, '_>,
    head: ScalarLoopHead,
    outer_loop: Option<ScalarLoopContext>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let context = builder
        .active_loop
        .take()
        .ok_or_else(|| invalid(span.clone()))?;
    if context.head != head {
        return Err(invalid(span));
    }
    if matches!(
        context.continuation,
        ScalarContinueDestination::Update { sites: Some(_), .. }
            | ScalarContinueDestination::Update { facts: Some(_), .. }
    ) {
        return Err(invalid(span));
    }
    builder.bind_transfers(context.breaks, span)?;
    builder.active_loop = outer_loop;
    Ok(())
}

pub(super) fn lower(
    builder: &mut ScalarBuilder<'_, '_>,
    kind: ScalarLoopTransfer,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let Some(context) = builder.active_loop.as_ref() else {
        let kind = match kind {
            ScalarLoopTransfer::Break => ScalarCompileErrorKind::IllegalBreak,
            ScalarLoopTransfer::Continue => ScalarCompileErrorKind::IllegalContinue,
        };
        return Err(scalar_compile_error::at(span, kind));
    };
    match kind {
        ScalarLoopTransfer::Continue => match &context.continuation {
            ScalarContinueDestination::Condition => {
                builder.emit(KernelInstruction::Jump(context.head.0), span)?;
            }
            ScalarContinueDestination::Update { .. } => {
                scalar_for_update::record_continue(builder, span)?;
            }
        },
        ScalarLoopTransfer::Break => {
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

fn invalid(span: Range<u64>) -> ScalarCompileError {
    scalar_compile_error::at(span, ScalarCompileErrorKind::InvalidLoopContext)
}
