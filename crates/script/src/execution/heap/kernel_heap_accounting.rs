// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_heap_error;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelHeapUsage, KernelResource};
use crate::heap::{HeapError, HeapWorkBudget, HeapWorkUsage};

impl KernelHeapContext<'_> {
    pub(super) fn charge(&mut self, steps: u64) -> Result<(), KernelErrorKind> {
        charge(&mut self.usage, self.budget.max_steps, steps)
    }

    pub(super) fn scratch(&mut self, bytes: u64) -> Result<(), KernelErrorKind> {
        let bytes = bytes
            .checked_add(self.literals.bytes)
            .and_then(|bytes| bytes.checked_add(self.array_owners.bytes))
            .ok_or(KernelErrorKind::SizeOverflow)?;
        check(
            KernelResource::HeapScratchBytes,
            bytes,
            self.budget.max_scratch_bytes,
        )?;
        self.usage.peak_scratch_bytes = self.usage.peak_scratch_bytes.max(bytes);
        Ok(())
    }

    pub(super) fn remaining(&self, root_bytes: u64) -> Result<HeapWorkBudget, KernelErrorKind> {
        Ok(HeapWorkBudget {
            max_steps: self
                .budget
                .max_steps
                .checked_sub(self.usage.steps)
                .ok_or(KernelErrorKind::InvalidExecutionState)?,
            max_scratch_bytes: self
                .budget
                .max_scratch_bytes
                .checked_sub(root_bytes)
                .and_then(|bytes| bytes.checked_sub(self.literals.bytes))
                .and_then(|bytes| bytes.checked_sub(self.array_owners.bytes))
                .ok_or(KernelErrorKind::InvalidExecutionState)?,
        })
    }

    pub(super) fn record(
        &mut self,
        usage: HeapWorkUsage,
        root_bytes: u64,
    ) -> Result<(), KernelErrorKind> {
        self.charge(usage.steps)?;
        self.scratch(
            root_bytes
                .checked_add(usage.scratch_bytes)
                .ok_or(KernelErrorKind::SizeOverflow)?,
        )
    }

    pub(super) fn record_error(
        &mut self,
        error: HeapError,
        root_bytes: u64,
    ) -> Result<KernelErrorKind, KernelErrorKind> {
        let base = self.usage.steps;
        self.record(error.usage, root_bytes)?;
        let mut kind = kernel_heap_error::map(error.kind);
        self.offset_error(&mut kind, base, root_bytes)?;
        Ok(kind)
    }

    pub(super) fn offset_error(
        &self,
        kind: &mut KernelErrorKind,
        base: u64,
        root_bytes: u64,
    ) -> Result<(), KernelErrorKind> {
        if let KernelErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        } = kind
        {
            match resource {
                KernelResource::HeapWorkSteps => {
                    *requested = base
                        .checked_add(*requested)
                        .ok_or(KernelErrorKind::SizeOverflow)?;
                    *maximum = self.budget.max_steps;
                }
                KernelResource::HeapScratchBytes => {
                    *requested = root_bytes
                        .checked_add(*requested)
                        .and_then(|bytes| bytes.checked_add(self.literals.bytes))
                        .and_then(|bytes| bytes.checked_add(self.array_owners.bytes))
                        .ok_or(KernelErrorKind::SizeOverflow)?;
                    *maximum = self.budget.max_scratch_bytes;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub(super) fn charge(
    usage: &mut KernelHeapUsage,
    maximum: u64,
    steps: u64,
) -> Result<(), KernelErrorKind> {
    let next = usage
        .steps
        .checked_add(steps)
        .ok_or(KernelErrorKind::SizeOverflow)?;
    check(KernelResource::HeapWorkSteps, next, maximum)?;
    usage.steps = next;
    Ok(())
}

fn check(resource: KernelResource, requested: u64, maximum: u64) -> Result<(), KernelErrorKind> {
    if requested > maximum {
        return Err(KernelErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        });
    }
    Ok(())
}
