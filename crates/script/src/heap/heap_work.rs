// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::heap_error;
use crate::heap::{HeapError, HeapErrorKind, HeapResource, HeapWorkBudget, HeapWorkUsage};

pub(super) struct HeapWork {
    budget: HeapWorkBudget,
    pub(super) usage: HeapWorkUsage,
}

impl HeapWork {
    pub(super) fn new(budget: HeapWorkBudget) -> Self {
        Self {
            budget,
            usage: HeapWorkUsage::default(),
        }
    }

    pub(super) fn charge(&mut self, steps: u64) -> Result<(), HeapErrorKind> {
        let next = self
            .usage
            .steps
            .checked_add(steps)
            .ok_or(HeapErrorKind::SizeOverflow)?;
        heap_error::check(HeapResource::WorkSteps, next, self.budget.max_steps)?;
        self.usage.steps = next;
        Ok(())
    }

    pub(super) fn scratch(&mut self, bytes: u64) -> Result<(), HeapErrorKind> {
        heap_error::check(
            HeapResource::ScratchBytes,
            bytes,
            self.budget.max_scratch_bytes,
        )?;
        self.usage.scratch_bytes = bytes;
        Ok(())
    }

    pub(super) fn error(&self, kind: HeapErrorKind) -> HeapError {
        HeapError {
            kind,
            usage: self.usage,
        }
    }
}
