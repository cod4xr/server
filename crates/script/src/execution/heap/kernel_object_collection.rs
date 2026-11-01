// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{kernel_object_error, kernel_object_roots};
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};

pub(super) fn collect(
    context: &mut KernelHeapContext<'_>,
    values: impl Iterator<Item = KernelValue> + Clone,
) -> Result<(), KernelErrorKind> {
    let roots = kernel_object_roots::gather(context, values)?;
    let object_collections = context
        .usage
        .object_collections
        .checked_add(1)
        .ok_or(KernelErrorKind::SizeOverflow)?;
    let collections = context
        .usage
        .collections
        .checked_add(1)
        .ok_or(KernelErrorKind::SizeOverflow)?;
    let budget = context.remaining(roots.bytes)?;
    let result = context
        .objects
        .as_mut()
        .ok_or(KernelErrorKind::InvalidExecutionState)?
        .reclaim(&roots.objects, budget);
    match result {
        Ok(result) => {
            context.record(result.usage, roots.bytes)?;
            context.usage.object_collections = object_collections;
        }
        Err(error) => return Err(kernel_object_error::record(context, error, roots.bytes)?),
    }
    // Object collection has completed and may persist if string collection fails.
    // Only unreachable graphs were removed; every live frame/host/cache root remains.
    let budget = context.remaining(roots.bytes)?;
    let result = context
        .objects
        .as_ref()
        .ok_or(KernelErrorKind::InvalidExecutionState)?
        .reclaim_strings(context.heap, &roots.strings, budget);
    match result {
        Ok(result) => {
            context.record(result.usage, roots.bytes)?;
            context.usage.collections = collections;
            Ok(())
        }
        Err(error) => Err(kernel_object_error::record(context, error, roots.bytes)?),
    }
}
