// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::{switch_nodes, switch_sort};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_switch::switch_context::SwitchCase;
use crate::compilation::function_switch::switch_error;
use crate::compilation::function_task::FunctionPatchSite;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;

pub(super) fn prepare(
    builder: &mut FunctionBuilder<'_, '_>,
    cases: &mut [SwitchCase],
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    switch_sort::prepare(builder, cases, span)
}

pub(super) fn emit(
    builder: &mut FunctionBuilder<'_, '_>,
    cases: &[SwitchCase],
    default: Option<usize>,
    breaks: Option<FunctionPatchSite>,
    span: Range<u64>,
) -> Result<Option<FunctionPatchSite>, FunctionCompileError> {
    let width = if builder.mode.bound() {
        switch_nodes::BOUND_WIDTH
    } else {
        switch_nodes::LEGACY_WIDTH
    };
    if !cases.is_empty() {
        let base = builder
            .instructions
            .len()
            .checked_add(1)
            .ok_or_else(|| overflow(span.clone()))?;
        let miss = address(base, cases.len(), width, span.clone())?;
        builder.emit(
            KernelInstruction::Jump(address(base, cases.len() / 2, width, span.clone())?),
            span.clone(),
        )?;
        for (index, case) in cases.iter().enumerate() {
            builder.ledger.charge(1, case.span.clone())?;
            let pc = address(base, index, width, case.span.clone())?;
            if builder.instructions.len() != pc {
                return Err(switch_error::invalid(case.span.clone()));
            }
            let target = case
                .target
                .ok_or_else(|| switch_error::invalid(case.span.clone()))?;
            let (left, right) = children(
                builder,
                index,
                cases.len(),
                base,
                miss,
                width,
                case.span.clone(),
            )?;
            // The complete footprint above proves each node's fixed pc offsets.
            switch_nodes::emit(
                builder,
                case.key,
                pc,
                target,
                (left, right),
                case.span.clone(),
            )?;
        }
        if builder.instructions.len() != miss {
            return Err(switch_error::invalid(span));
        }
    }
    builder.emit(KernelInstruction::Discard, span.clone())?;
    if let Some(default) = default {
        builder.emit(KernelInstruction::Jump(default), span)?;
        Ok(breaks)
    } else {
        builder.transfer_jump(breaks, span).map(Some)
    }
}

fn children(
    builder: &mut FunctionBuilder<'_, '_>,
    index: usize,
    count: usize,
    base: usize,
    miss: usize,
    width: usize,
    span: Range<u64>,
) -> Result<(usize, usize), FunctionCompileError> {
    let (mut low, mut high) = (0, count);
    while low < high {
        builder.ledger.charge(1, span.clone())?;
        let middle = low + (high - low) / 2;
        if index == middle {
            let left = if low < middle {
                address(base, low + (middle - low) / 2, width, span.clone())?
            } else {
                miss
            };
            let next = middle + 1; // middle < high <= count, so this cannot wrap.
            let right = if next < high {
                address(base, next + (high - next) / 2, width, span.clone())?
            } else {
                miss
            };
            return Ok((left, right));
        }
        if index < middle {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    Err(switch_error::invalid(span))
}

fn address(
    base: usize,
    index: usize,
    width: usize,
    span: Range<u64>,
) -> Result<usize, FunctionCompileError> {
    index
        .checked_mul(width)
        .and_then(|offset| base.checked_add(offset))
        .ok_or_else(|| overflow(span))
}

fn overflow(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::SizeOverflow)
}
