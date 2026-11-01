// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};

/// Flat source-ordered facts; absence denotes an uninitialized later binding.
pub(super) struct ScalarInitializationSnapshot {
    initialized: Vec<bool>,
    reachable: bool,
}

#[derive(Clone, Copy)]
enum SnapshotAction {
    Restore,
    Merge,
}

pub(super) fn capture(
    builder: &mut ScalarBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<ScalarInitializationSnapshot, ScalarCompileError> {
    let count = builder
        .locals
        .as_ref()
        .ok_or_else(|| invalid(span.clone()))?
        .names
        .len();
    let work = (count as u64).checked_mul(2).ok_or_else(|| {
        scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
    })?;
    builder.ledger.charge(work, span.clone())?;
    builder.admit_control(count as u64, span.clone())?;
    let mut initialized = Vec::new();
    initialized.try_reserve_exact(count).map_err(|_| {
        scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::AllocationFailed)
    })?;
    initialized.resize(count, false);
    let locals = builder
        .locals
        .as_ref()
        .ok_or_else(|| invalid(span.clone()))?;
    for slot in locals.names.values() {
        let flag = initialized
            .get_mut(slot.index)
            .ok_or_else(|| invalid(span.clone()))?;
        *flag = slot.initialized;
    }
    Ok(ScalarInitializationSnapshot {
        initialized,
        reachable: builder.reachable,
    })
}

pub(super) fn restore(
    builder: &mut ScalarBuilder<'_, '_>,
    snapshot: ScalarInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    apply(builder, &snapshot, SnapshotAction::Restore, span.clone())?;
    release(builder, snapshot, span)
}

pub(super) fn restore_from(
    builder: &mut ScalarBuilder<'_, '_>,
    snapshot: &ScalarInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    apply(builder, snapshot, SnapshotAction::Restore, span)
}

pub(super) fn merge(
    builder: &mut ScalarBuilder<'_, '_>,
    snapshot: ScalarInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    apply(builder, &snapshot, SnapshotAction::Merge, span.clone())?;
    release(builder, snapshot, span)
}

fn apply(
    builder: &mut ScalarBuilder<'_, '_>,
    snapshot: &ScalarInitializationSnapshot,
    action: SnapshotAction,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
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
    for slot in locals.names.values_mut() {
        let incoming = snapshot
            .initialized
            .get(slot.index)
            .copied()
            .unwrap_or(false);
        slot.initialized = match action {
            SnapshotAction::Merge if current_reachable => {
                slot.initialized && (!snapshot.reachable || incoming)
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
fn release(
    builder: &mut ScalarBuilder<'_, '_>,
    snapshot: ScalarInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    builder
        .ledger
        .release_control(snapshot.initialized.len() as u64, span)
}

fn invalid(span: Range<u64>) -> ScalarCompileError {
    scalar_compile_error::at(span, ScalarCompileErrorKind::InvalidControlState)
}
