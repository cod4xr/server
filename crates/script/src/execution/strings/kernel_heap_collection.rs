// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::strings::KernelHeapContext;
use crate::execution::{KernelErrorKind, KernelValue};
use crate::heap::StringId;

pub(super) fn collect(
    context: &mut KernelHeapContext<'_>,
    values: impl Iterator<Item = KernelValue> + Clone,
) -> Result<(), KernelErrorKind> {
    let mut count = context.external_roots.len() as u64;
    for value in values.clone() {
        context.charge(1)?;
        if matches!(value, KernelValue::String(_)) {
            count = count.checked_add(1).ok_or(KernelErrorKind::SizeOverflow)?;
        }
    }
    let root_bytes = count
        .checked_mul(size_of::<StringId>() as u64)
        .ok_or(KernelErrorKind::SizeOverflow)?;
    let scratch = root_bytes
        .checked_add(context.heap.usage().slots)
        .ok_or(KernelErrorKind::SizeOverflow)?;
    context.scratch(scratch)?;
    let count = usize::try_from(count).map_err(|_| KernelErrorKind::SizeOverflow)?;
    let mut roots = Vec::new();
    roots
        .try_reserve_exact(count)
        .map_err(|_| KernelErrorKind::AllocationFailed)?;
    for value in values {
        context.charge(1)?;
        if let KernelValue::String(id) = value {
            roots.push(id);
        }
    }
    for index in 0..context.external_roots.len() {
        context.charge(1)?;
        roots.push(context.external_roots[index]);
    }
    if roots.len() != count {
        return Err(KernelErrorKind::InvalidExecutionState);
    }
    let next = context
        .usage
        .collections
        .checked_add(1)
        .ok_or(KernelErrorKind::SizeOverflow)?;
    let result = context.heap.reclaim(&roots, context.remaining(root_bytes)?);
    match result {
        Ok(result) => {
            context.record(result.usage, root_bytes)?;
            context.usage.collections = next;
            Ok(())
        }
        Err(error) => Err(context.record_error(error, root_bytes)?),
    }
}
