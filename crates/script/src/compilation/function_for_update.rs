// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_initialization::{self, FunctionInitializationSnapshot};
use crate::compilation::function_loop_transfer::FunctionContinueDestination;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

pub(super) fn record_continue(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let FunctionContinueDestination::Update { facts, sites } = destination(builder, span.clone())?
    else {
        return Err(invalid(span));
    };
    let previous = facts.take();
    let chain = sites.take();
    if let Some(previous) = previous {
        function_initialization::merge(builder, previous, span.clone())?;
    }
    let merged = function_initialization::capture(builder, span.clone())?;
    let site = builder.transfer_jump(chain, span.clone())?;
    let FunctionContinueDestination::Update { facts, sites } = destination(builder, span.clone())?
    else {
        return Err(invalid(span));
    };
    *facts = Some(merged);
    *sites = Some(site);
    Ok(())
}

pub(super) fn prepare(
    builder: &mut FunctionBuilder<'_, '_>,
    incoming: &FunctionInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let FunctionContinueDestination::Update { facts, sites } = destination(builder, span.clone())?
    else {
        return Err(invalid(span));
    };
    let previous = facts.take();
    let chain = sites.take();
    if let Some(previous) = previous {
        function_initialization::merge(builder, previous, span.clone())?;
    }
    if !builder.reachable {
        function_initialization::restore_from(builder, incoming, span.clone())?;
        builder.reachable = true;
    }
    builder.bind_transfers(chain, span)
}

fn destination<'builder>(
    builder: &'builder mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<&'builder mut FunctionContinueDestination, FunctionCompileError> {
    builder
        .active_loop
        .as_mut()
        .map(|context| &mut context.continuation)
        .ok_or_else(|| invalid(span))
}

fn invalid(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::InvalidLoopContext)
}
