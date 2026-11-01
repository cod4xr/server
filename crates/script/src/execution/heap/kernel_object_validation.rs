// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelErrorKind, KernelHeapContext};
use crate::heap::ObjectId;

pub(super) fn binding(context: &mut KernelHeapContext<'_>) -> Result<(), KernelErrorKind> {
    if context.objects.is_some() {
        context.charge(1)?;
        context
            .objects
            .as_ref()
            .ok_or(KernelErrorKind::InvalidExecutionState)?
            .validate_strings(context.heap)
            .map_err(KernelErrorKind::ObjectHeap)?;
    }
    Ok(())
}

pub(super) fn reference(
    context: &KernelHeapContext<'_>,
    id: ObjectId,
) -> Result<(), KernelErrorKind> {
    context
        .objects
        .as_ref()
        .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
        .field_count(id)
        .map_err(KernelErrorKind::ObjectHeap)?;
    Ok(())
}
