// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_switch::switch_context::SwitchCase;
use crate::compilation::function_switch::switch_error;
use crate::compilation::function_task::FunctionPatchSite;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::{KernelBranchCondition, KernelInstruction, KernelValue};
use crate::legacy::{LegacyComparison, LegacyNumber};

const NODE_INSTRUCTIONS: usize = 11;

pub(super) fn prepare(
    builder: &mut FunctionBuilder<'_, '_>,
    cases: &mut [SwitchCase],
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let count = cases.len() as u64;
    // Explicit logical sorting charge, distinct from measured CPU time/allocator work.
    let levels = u64::from(count.checked_ilog2().unwrap_or(0)) + 1;
    let work = count
        .checked_mul(levels)
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| overflow(span.clone()))?;
    builder.ledger.charge(work, span.clone())?;
    cases.sort_unstable_by_key(|case| case.key);
    for pair in cases.windows(2) {
        builder.ledger.charge(1, pair[1].span.clone())?;
        if pair[0].key == pair[1].key {
            return Err(function_compile_error::at(
                pair[1].span.clone(),
                FunctionCompileErrorKind::DuplicateCase,
            ));
        }
    }
    Ok(())
}

pub(super) fn emit(
    builder: &mut FunctionBuilder<'_, '_>,
    cases: &[SwitchCase],
    default: Option<usize>,
    breaks: Option<FunctionPatchSite>,
    span: Range<u64>,
) -> Result<Option<FunctionPatchSite>, FunctionCompileError> {
    if !cases.is_empty() {
        let base = builder
            .instructions
            .len()
            .checked_add(1)
            .ok_or_else(|| overflow(span.clone()))?;
        let miss = address(base, cases.len(), span.clone())?;
        builder.emit(
            KernelInstruction::Jump(address(base, cases.len() / 2, span.clone())?),
            span.clone(),
        )?;
        for (index, case) in cases.iter().enumerate() {
            builder.ledger.charge(1, case.span.clone())?;
            let pc = address(base, index, case.span.clone())?;
            if builder.instructions.len() != pc {
                return Err(switch_error::invalid(case.span.clone()));
            }
            let target = case
                .target
                .ok_or_else(|| switch_error::invalid(case.span.clone()))?;
            let (left, right) =
                children(builder, index, cases.len(), base, miss, case.span.clone())?;
            let constant =
                KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Integer(case.key)));
            // The checked complete footprint above proves pc+6 is representable.
            for instruction in [
                KernelInstruction::Duplicate,
                constant,
                KernelInstruction::Compare(LegacyComparison::Equal),
                KernelInstruction::Branch {
                    condition: KernelBranchCondition::IfFalse,
                    target: pc + 6,
                },
                KernelInstruction::Discard,
                KernelInstruction::Jump(target),
                KernelInstruction::Duplicate,
                constant,
                KernelInstruction::Compare(LegacyComparison::Less),
                KernelInstruction::Branch {
                    condition: KernelBranchCondition::IfTrue,
                    target: left,
                },
                KernelInstruction::Jump(right),
            ] {
                builder.emit(instruction, case.span.clone())?;
            }
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
    span: Range<u64>,
) -> Result<(usize, usize), FunctionCompileError> {
    let (mut low, mut high) = (0, count);
    while low < high {
        builder.ledger.charge(1, span.clone())?;
        let middle = low + (high - low) / 2;
        if index == middle {
            let left = if low < middle {
                address(base, low + (middle - low) / 2, span.clone())?
            } else {
                miss
            };
            let next = middle + 1; // middle < high <= count, so this cannot wrap.
            let right = if next < high {
                address(base, next + (high - next) / 2, span.clone())?
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

fn address(base: usize, index: usize, span: Range<u64>) -> Result<usize, FunctionCompileError> {
    index
        .checked_mul(NODE_INSTRUCTIONS)
        .and_then(|offset| base.checked_add(offset))
        .ok_or_else(|| overflow(span))
}

fn overflow(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::SizeOverflow)
}
