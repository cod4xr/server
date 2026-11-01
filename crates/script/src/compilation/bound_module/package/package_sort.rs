// SPDX-License-Identifier: AGPL-3.0-or-later
use std::cmp::Ordering;

use crate::binding::CanonicalScriptPath;
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::package::ScriptPackageError;
use crate::compilation::bound_module::package::package_catalogue;
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

pub(super) fn build(
    names: &[CanonicalScriptPath],
    ledger: &mut ModuleCompilationLedger,
) -> Result<Vec<usize>, ScriptPackageError> {
    let count = names.len();
    let scratch = package_catalogue::bytes(count as u64, size_of::<usize>())
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    ledger
        .temporary(scratch, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    ledger
        .charge(
            (count as u64).checked_mul(2).ok_or_else(|| {
                package_catalogue::locate(
                    None,
                    function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow),
                    ledger,
                )
            })?,
            0..0,
        )
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    let mut order = module_storage::reserve(count as u64, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    let mut scratch = module_storage::reserve(count as u64, 0..0)
        .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
    order.extend(0..count);
    scratch.resize(count, 0);
    let mut width = 1;
    while width < count {
        merge(names, &order, &mut scratch, width, ledger)
            .map_err(|cause| package_catalogue::locate(None, cause, ledger))?;
        std::mem::swap(&mut order, &mut scratch);
        width = width.checked_mul(2).ok_or_else(|| {
            package_catalogue::locate(
                None,
                function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow),
                ledger,
            )
        })?;
    }
    for pair in order.windows(2) {
        let left = names[pair[0]].as_bytes();
        let right = names[pair[1]].as_bytes();
        if compare(left, right, ledger)
            .map_err(|cause| package_catalogue::locate(None, cause, ledger))?
            == Ordering::Equal
        {
            return Err(package_catalogue::locate(
                Some(pair[1]),
                function_compile_error::at(0..0, FunctionCompileErrorKind::DuplicateModule),
                ledger,
            ));
        }
    }
    Ok(order)
}

fn merge(
    names: &[CanonicalScriptPath],
    source: &[usize],
    target: &mut [usize],
    width: usize,
    ledger: &mut ModuleCompilationLedger,
) -> Result<(), FunctionCompileError> {
    let mut start = 0;
    while start < source.len() {
        let middle = start.saturating_add(width).min(source.len());
        let end = middle.saturating_add(width).min(source.len());
        let (mut left, mut right) = (start, middle);
        for output in &mut target[start..end] {
            ledger.charge(1, 0..0)?;
            let take_left = right == end
                || (left < middle
                    && compare(
                        names[source[left]].as_bytes(),
                        names[source[right]].as_bytes(),
                        ledger,
                    )? != Ordering::Greater);
            if take_left {
                *output = source[left];
                left += 1;
            } else {
                *output = source[right];
                right += 1;
            }
        }
        start = end;
    }
    Ok(())
}

fn compare(
    left: &[u8],
    right: &[u8],
    ledger: &mut ModuleCompilationLedger,
) -> Result<Ordering, FunctionCompileError> {
    let visits = (left.len() as u64)
        .checked_add(right.len() as u64)
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or_else(|| function_compile_error::at(0..0, FunctionCompileErrorKind::SizeOverflow))?;
    ledger.charge(visits, 0..0)?;
    Ok(left.cmp(right))
}
