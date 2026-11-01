// SPDX-License-Identifier: AGPL-3.0-or-later
use super::switch_context::SwitchCase;
use super::switch_error;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::execution::KernelSwitchKey;
use std::cmp::Ordering;
use std::ops::Range;

pub(super) fn prepare(
    builder: &mut FunctionBuilder<'_, '_>,
    cases: &mut [SwitchCase],
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    // Iterative in-place heapsort makes every comparison/swap fallible and
    // metered, including long equal prefixes. No sorting scratch is allocated.
    let count = cases.len();
    for root in (0..count / 2).rev() {
        sift(builder, cases, root, count, span.clone())?;
    }
    for end in (1..cases.len()).rev() {
        builder.ledger.charge(1, span.clone())?;
        cases.swap(0, end);
        sift(builder, cases, 0, end, span.clone())?;
    }
    for pair in cases.windows(2) {
        if key_order(builder, pair[0].key, pair[1].key, pair[1].span.clone())? == Ordering::Equal {
            return Err(function_compile_error::at(
                pair[1].span.clone(),
                FunctionCompileErrorKind::DuplicateCase,
            ));
        }
    }
    Ok(())
}

fn sift(
    builder: &mut FunctionBuilder<'_, '_>,
    cases: &mut [SwitchCase],
    mut root: usize,
    end: usize,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    loop {
        builder.ledger.charge(1, span.clone())?;
        let child = root
            .checked_mul(2)
            .and_then(|index| index.checked_add(1))
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        if child >= end {
            return Ok(());
        }
        let mut largest = child;
        let right = child.checked_add(1).ok_or_else(|| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
        })?;
        if right < end
            && order(builder, &cases[child], &cases[right], span.clone())? == Ordering::Less
        {
            largest = right;
        }
        if order(builder, &cases[root], &cases[largest], span.clone())? != Ordering::Less {
            return Ok(());
        }
        builder.ledger.charge(1, span.clone())?;
        cases.swap(root, largest);
        root = largest;
    }
}

fn order(
    builder: &mut FunctionBuilder<'_, '_>,
    left: &SwitchCase,
    right: &SwitchCase,
    span: Range<u64>,
) -> Result<Ordering, FunctionCompileError> {
    let order = key_order(builder, left.key, right.key, span.clone())?;
    if order != Ordering::Equal {
        return Ok(order);
    }
    builder.ledger.charge(1, span)?;
    // Equal decoded keys retain source order for deterministic duplicate spans.
    Ok(left.span.start.cmp(&right.span.start))
}

fn key_order(
    builder: &mut FunctionBuilder<'_, '_>,
    left: KernelSwitchKey,
    right: KernelSwitchKey,
    span: Range<u64>,
) -> Result<Ordering, FunctionCompileError> {
    builder.ledger.charge(1, span.clone())?;
    match (left, right) {
        (KernelSwitchKey::Integer(left), KernelSwitchKey::Integer(right)) => Ok(left.cmp(&right)),
        (KernelSwitchKey::Integer(_), KernelSwitchKey::StringLabel(_)) => Ok(Ordering::Less),
        (KernelSwitchKey::StringLabel(_), KernelSwitchKey::Integer(_)) => Ok(Ordering::Greater),
        (KernelSwitchKey::StringLabel(left), KernelSwitchKey::StringLabel(right)) => {
            builder.ledger.charge(2, span.clone())?;
            let left = builder
                .switch_labels
                .get(left)
                .ok_or_else(|| switch_error::invalid(span.clone()))?;
            let right = builder
                .switch_labels
                .get(right)
                .ok_or_else(|| switch_error::invalid(span.clone()))?;
            builder
                .ledger
                .charge(left.len().min(right.len()) as u64 + 1, span)?;
            Ok(left.cmp(right))
        }
    }
}
