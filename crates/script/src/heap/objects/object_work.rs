// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::heap_work::HeapWork;
use crate::heap::objects::{ObjectHeapError, ObjectHeapErrorKind, ObjectResource};
use crate::heap::{HeapErrorKind, HeapResource, HeapWorkBudget, HeapWorkUsage};

pub(super) struct ObjectWork {
    pub(super) budget: HeapWorkBudget,
    ledger: HeapWork,
}

impl ObjectWork {
    pub(super) fn new(budget: HeapWorkBudget) -> Self {
        Self {
            budget,
            ledger: HeapWork::new(budget),
        }
    }

    pub(super) fn usage(&self) -> HeapWorkUsage {
        self.ledger.usage
    }

    pub(super) fn charge(&mut self, steps: u64) -> Result<(), ObjectHeapErrorKind> {
        self.ledger.charge(steps).map_err(map)
    }

    pub(super) fn scratch(&mut self, bytes: u64) -> Result<(), ObjectHeapErrorKind> {
        self.ledger
            .scratch(bytes.max(self.ledger.usage.scratch_bytes))
            .map_err(map)
    }

    pub(super) fn error(&self, kind: ObjectHeapErrorKind) -> ObjectHeapError {
        ObjectHeapError {
            kind,
            usage: self.usage(),
        }
    }
}

fn map(kind: HeapErrorKind) -> ObjectHeapErrorKind {
    match kind {
        HeapErrorKind::SizeOverflow => ObjectHeapErrorKind::SizeOverflow,
        HeapErrorKind::BudgetExceeded {
            resource: HeapResource::WorkSteps,
            requested,
            maximum,
        } => ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::WorkSteps,
            requested,
            maximum,
        },
        HeapErrorKind::BudgetExceeded {
            resource: HeapResource::ScratchBytes,
            requested,
            maximum,
        } => ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::ScratchBytes,
            requested,
            maximum,
        },
        _ => ObjectHeapErrorKind::InvalidState,
    }
}
