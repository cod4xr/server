// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_heap_error;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelResource};
use crate::heap::{ObjectHeapError, ObjectHeapErrorKind, ObjectResource};

pub(super) fn record(
    context: &mut KernelHeapContext<'_>,
    error: ObjectHeapError,
    roots: u64,
) -> Result<KernelErrorKind, KernelErrorKind> {
    let base = context.usage.steps;
    context.record(error.usage, roots)?;
    let mut kind = map(error.kind);
    context.offset_error(&mut kind, base, roots)?;
    Ok(kind)
}

fn map(kind: ObjectHeapErrorKind) -> KernelErrorKind {
    match kind {
        ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::WorkSteps,
            requested,
            maximum,
        } => KernelErrorKind::BudgetExceeded {
            resource: KernelResource::HeapWorkSteps,
            requested,
            maximum,
        },
        ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::ScratchBytes,
            requested,
            maximum,
        } => KernelErrorKind::BudgetExceeded {
            resource: KernelResource::HeapScratchBytes,
            requested,
            maximum,
        },
        ObjectHeapErrorKind::StringHeap(kind) => kernel_heap_error::map(kind),
        kind => KernelErrorKind::ObjectHeap(kind),
    }
}
