// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

/// Flat source-ordered facts; absence denotes an uninitialized later binding.
pub(super) struct FunctionInitializationSnapshot {
    initialized: Vec<bool>,
    reachable: bool,
}

impl FunctionInitializationSnapshot {
    pub(super) fn reachable(&self) -> bool {
        self.reachable
    }
}

#[derive(Clone, Copy)]
enum SnapshotAction {
    Restore,
    Merge,
}

pub(super) fn capture(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<FunctionInitializationSnapshot, FunctionCompileError> {
    let count = builder
        .locals
        .as_ref()
        .ok_or_else(|| invalid(span.clone()))?
        .names
        .len();
    let work = (count as u64).checked_mul(2).ok_or_else(|| {
        function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
    })?;
    builder.ledger.charge(work, span.clone())?;
    builder.admit_control(count as u64, span.clone())?;
    let mut initialized = Vec::new();
    initialized.try_reserve_exact(count).map_err(|_| {
        function_compile_error::at(span.clone(), FunctionCompileErrorKind::AllocationFailed)
    })?;
    let locals = builder
        .locals
        .as_ref()
        .ok_or_else(|| invalid(span.clone()))?;
    initialized.extend_from_slice(&locals.initialized);
    Ok(FunctionInitializationSnapshot {
        initialized,
        reachable: builder.reachable,
    })
}

pub(super) fn restore(
    builder: &mut FunctionBuilder<'_, '_>,
    snapshot: FunctionInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    apply(builder, &snapshot, SnapshotAction::Restore, span.clone())?;
    release(builder, snapshot, span)
}

pub(super) fn restore_from(
    builder: &mut FunctionBuilder<'_, '_>,
    snapshot: &FunctionInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    apply(builder, snapshot, SnapshotAction::Restore, span)
}

pub(super) fn merge(
    builder: &mut FunctionBuilder<'_, '_>,
    snapshot: FunctionInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    apply(builder, &snapshot, SnapshotAction::Merge, span.clone())?;
    release(builder, snapshot, span)
}

pub(super) fn merge_from(
    builder: &mut FunctionBuilder<'_, '_>,
    snapshot: &FunctionInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    apply(builder, snapshot, SnapshotAction::Merge, span)
}

fn apply(
    builder: &mut FunctionBuilder<'_, '_>,
    snapshot: &FunctionInitializationSnapshot,
    action: SnapshotAction,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let count = builder
        .locals
        .as_ref()
        .ok_or_else(|| invalid(span.clone()))?
        .names
        .len();
    builder.ledger.charge(count as u64, span.clone())?;
    let current_reachable = builder.reachable;
    let locals = builder
        .locals
        .as_mut()
        .ok_or_else(|| invalid(span.clone()))?;
    for (index, flag) in locals.initialized.iter_mut().enumerate() {
        let incoming = snapshot.initialized.get(index).copied().unwrap_or(false);
        *flag = match action {
            SnapshotAction::Merge if current_reachable => {
                *flag && (!snapshot.reachable || incoming)
            }
            SnapshotAction::Restore | SnapshotAction::Merge => incoming,
        };
    }
    builder.reachable = match action {
        SnapshotAction::Merge => current_reachable || snapshot.reachable,
        SnapshotAction::Restore => snapshot.reachable,
    };
    Ok(())
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Consuming the snapshot releases its admitted flag payload exactly once."
)]
pub(super) fn release(
    builder: &mut FunctionBuilder<'_, '_>,
    snapshot: FunctionInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    builder
        .ledger
        .release_control(snapshot.initialized.len() as u64, span)
}

fn invalid(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::InvalidControlState)
}
