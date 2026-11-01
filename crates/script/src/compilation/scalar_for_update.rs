// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_initialization::{self, ScalarInitializationSnapshot};
use crate::compilation::scalar_loop_transfer::ScalarContinueDestination;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};

pub(super) fn record_continue(
    builder: &mut ScalarBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let ScalarContinueDestination::Update { facts, sites } = destination(builder, span.clone())?
    else {
        return Err(invalid(span));
    };
    let previous = facts.take();
    let chain = sites.take();
    if let Some(previous) = previous {
        scalar_initialization::merge(builder, previous, span.clone())?;
    }
    let merged = scalar_initialization::capture(builder, span.clone())?;
    let site = builder.transfer_jump(chain, span.clone())?;
    let ScalarContinueDestination::Update { facts, sites } = destination(builder, span.clone())?
    else {
        return Err(invalid(span));
    };
    *facts = Some(merged);
    *sites = Some(site);
    Ok(())
}

pub(super) fn prepare(
    builder: &mut ScalarBuilder<'_, '_>,
    incoming: &ScalarInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let ScalarContinueDestination::Update { facts, sites } = destination(builder, span.clone())?
    else {
        return Err(invalid(span));
    };
    let previous = facts.take();
    let chain = sites.take();
    if let Some(previous) = previous {
        scalar_initialization::merge(builder, previous, span.clone())?;
    }
    if !builder.reachable {
        scalar_initialization::restore_from(builder, incoming, span.clone())?;
        builder.reachable = true;
    }
    builder.bind_transfers(chain, span)
}

fn destination<'builder>(
    builder: &'builder mut ScalarBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<&'builder mut ScalarContinueDestination, ScalarCompileError> {
    builder
        .active_loop
        .as_mut()
        .map(|context| &mut context.continuation)
        .ok_or_else(|| invalid(span))
}

fn invalid(span: Range<u64>) -> ScalarCompileError {
    scalar_compile_error::at(span, ScalarCompileErrorKind::InvalidLoopContext)
}
