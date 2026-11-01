// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelErrorKind, KernelResource};
use crate::heap::{HeapErrorKind, HeapResource};

pub(super) fn map(kind: HeapErrorKind) -> KernelErrorKind {
    match kind {
        HeapErrorKind::ForeignReference => KernelErrorKind::ForeignStringReference,
        HeapErrorKind::StaleReference => KernelErrorKind::StaleStringReference,
        HeapErrorKind::IdentityExhausted => KernelErrorKind::HeapIdentityExhausted,
        HeapErrorKind::SizeOverflow => KernelErrorKind::SizeOverflow,
        HeapErrorKind::AllocationFailed => KernelErrorKind::AllocationFailed,
        HeapErrorKind::InvalidState => KernelErrorKind::InvalidExecutionState,
        HeapErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        } => {
            let resource = match resource {
                HeapResource::LiveStrings => KernelResource::StringValues,
                HeapResource::PayloadBytes => KernelResource::StringPayloadBytes,
                HeapResource::Slots => KernelResource::StringSlots,
                HeapResource::MetadataBytes => KernelResource::StringMetadataBytes,
                HeapResource::SingleStringBytes => KernelResource::SingleStringBytes,
                HeapResource::WorkSteps => KernelResource::HeapWorkSteps,
                HeapResource::ScratchBytes => KernelResource::HeapScratchBytes,
            };
            KernelErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum,
            }
        }
    }
}
