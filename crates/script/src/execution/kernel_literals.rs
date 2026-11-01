// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_verification_ledger::KernelVerificationLedger;
use crate::execution::{KernelError, KernelErrorKind};

// Callers precharge all descriptor visits before inspecting this borrowed table.
pub(super) fn retained_bytes(literals: &[Vec<u8>]) -> Result<u64, KernelError> {
    let mut bytes = (literals.len() as u64)
        .checked_mul(size_of::<Vec<u8>>() as u64)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
    for literal in literals {
        bytes = bytes
            .checked_add(literal.len() as u64)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
    }
    Ok(bytes)
}

pub(super) fn copy(
    literals: &[Vec<u8>],
    ledger: &mut KernelVerificationLedger,
) -> Result<Vec<Vec<u8>>, KernelError> {
    ledger.charge(literals.len() as u64, None)?;
    ledger.retain_strings(literals.len() as u64, retained_bytes(literals)?)?;
    copy_rows(literals, ledger)
}

pub(super) fn copy_fields(
    names: &[Vec<u8>],
    ledger: &mut KernelVerificationLedger,
) -> Result<Vec<Vec<u8>>, KernelError> {
    ledger.charge(names.len() as u64, None)?;
    ledger.retain_fields(names.len() as u64, retained_bytes(names)?)?;
    copy_rows(names, ledger)
}

pub(super) fn copy_switch_labels(
    labels: &[Vec<u8>],
    ledger: &mut KernelVerificationLedger,
) -> Result<Vec<Vec<u8>>, KernelError> {
    ledger.charge(labels.len() as u64, None)?;
    ledger.retain_switch_labels(labels.len() as u64, retained_bytes(labels)?)?;
    copy_rows(labels, ledger)
}

fn copy_rows(
    literals: &[Vec<u8>],
    ledger: &mut KernelVerificationLedger,
) -> Result<Vec<Vec<u8>>, KernelError> {
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(literals.len())
        .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
    for literal in literals {
        let work = (literal.len() as u64)
            .checked_add(1)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        ledger.charge(work, None)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(literal.len())
            .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
        bytes.extend_from_slice(literal);
        owned.push(bytes);
    }
    Ok(owned)
}
