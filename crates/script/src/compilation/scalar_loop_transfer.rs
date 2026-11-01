// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_task::{ScalarLoopHead, ScalarPatchSite};
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::KernelInstruction;

pub(super) struct ScalarWhileContext {
    head: ScalarLoopHead,
    breaks: Option<ScalarPatchSite>,
}

#[derive(Clone, Copy)]
pub(super) enum ScalarLoopTransfer {
    Break,
    Continue,
}

pub(super) fn enter(
    builder: &mut ScalarBuilder<'_, '_>,
    head: ScalarLoopHead,
) -> Option<ScalarWhileContext> {
    builder
        .active_loop
        .replace(ScalarWhileContext { head, breaks: None })
}

pub(super) fn leave(
    builder: &mut ScalarBuilder<'_, '_>,
    head: ScalarLoopHead,
    outer_loop: Option<ScalarWhileContext>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let context = builder
        .active_loop
        .take()
        .ok_or_else(|| invalid(span.clone()))?;
    if context.head != head {
        return Err(invalid(span));
    }
    builder.bind_breaks(context.breaks, span)?;
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
        ScalarLoopTransfer::Continue => {
            builder.emit(KernelInstruction::Jump(context.head.0), span)?;
        }
        ScalarLoopTransfer::Break => {
            let chain = builder
                .active_loop
                .as_mut()
                .ok_or_else(|| invalid(span.clone()))?
                .breaks
                .take();
            let site = builder.break_jump(chain, span.clone())?;
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
