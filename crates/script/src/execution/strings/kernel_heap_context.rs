// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::strings::KernelHeapUsage;
use crate::execution::strings::kernel_heap_error;
use crate::execution::strings::kernel_literal_cache::KernelLiteralCache;
use crate::execution::{KernelErrorKind, KernelResource, KernelValue};
use crate::heap::{
    HeapError, HeapErrorKind, HeapWorkBudget, HeapWorkUsage, StringHeap, StringHeapUsage, StringId,
};

/// Caller-owned string execution context shared by the entire iterative call tree.
///
/// Copied IDs do not own storage. Supply all host handles that must survive VM
/// overwrites as external roots. Returned IDs stay in this borrowed heap and need
/// rooting in subsequent invocations. Invocation failures can retain allocations
/// or successful reclamation; individual store operations remain atomic.
/// Work accumulates when this context is reused; it never resets per callee.
/// Each invocation prepares a fresh admitted literal cache; cached-only strings
/// remain roots until the next invocation. Cache, root IDs and mark flags share
/// the supplied scratch ceiling. Dropping a cache does not drop heap payload.
pub struct KernelHeapContext<'heap> {
    pub(super) literals: KernelLiteralCache,
    pub(super) heap: &'heap mut StringHeap,
    pub(super) external_roots: &'heap [StringId],
    pub(super) budget: HeapWorkBudget,
    pub(super) usage: KernelHeapUsage,
}

impl<'heap> KernelHeapContext<'heap> {
    /// Borrows storage and a complete external root inventory without allocation.
    /// Entry validation charges/checks roots before allocating execution frames.
    #[must_use]
    pub fn new(
        heap: &'heap mut StringHeap,
        external_roots: &'heap [StringId],
        budget: HeapWorkBudget,
    ) -> Self {
        Self {
            literals: KernelLiteralCache::empty(),
            heap,
            external_roots,
            budget,
            usage: KernelHeapUsage::default(),
        }
    }

    /// Returns charged global work/collection/scratch accounting, including failures.
    #[must_use]
    pub fn usage(&self) -> KernelHeapUsage {
        self.usage
    }

    /// Returns retained logical string-store usage without payload or identities.
    #[must_use]
    pub fn heap_usage(&self) -> StringHeapUsage {
        self.heap.usage()
    }

    /// Borrows explicit result bytes for a checked ID; this host read is not VM work.
    ///
    /// # Errors
    /// Rejects stale/foreign handles. No mutation or encoding conversion occurs.
    pub fn get(&self, id: StringId) -> Result<&[u8], HeapErrorKind> {
        self.heap.get(id)
    }

    pub(in crate::execution) fn validate(
        &mut self,
        arguments: &[KernelValue],
    ) -> Result<(), KernelErrorKind> {
        for value in arguments {
            self.charge(1)?;
            if let Some(id) = value.string_id() {
                self.heap.get(id).map_err(kernel_heap_error::map)?;
            }
        }
        for index in 0..self.external_roots.len() {
            self.charge(1)?;
            self.heap
                .get(self.external_roots[index])
                .map_err(kernel_heap_error::map)?;
        }
        Ok(())
    }

    pub(super) fn charge(&mut self, steps: u64) -> Result<(), KernelErrorKind> {
        let next = self
            .usage
            .steps
            .checked_add(steps)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        check(KernelResource::HeapWorkSteps, next, self.budget.max_steps)?;
        self.usage.steps = next;
        Ok(())
    }

    pub(super) fn scratch(&mut self, bytes: u64) -> Result<(), KernelErrorKind> {
        let bytes = bytes
            .checked_add(self.literals.bytes)
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
        if let KernelErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        } = &mut kind
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
                        .ok_or(KernelErrorKind::SizeOverflow)?;
                    *maximum = self.budget.max_scratch_bytes;
                }
                _ => {}
            }
        }
        Ok(kind)
    }

    pub(super) fn equal(
        &mut self,
        left: StringId,
        right: StringId,
    ) -> Result<bool, KernelErrorKind> {
        self.charge(4)?;
        if left == right {
            self.heap.get(left).map_err(kernel_heap_error::map)?;
            return Ok(true);
        }
        let left_length = self.heap.get(left).map_err(kernel_heap_error::map)?.len();
        let right_length = self.heap.get(right).map_err(kernel_heap_error::map)?.len();
        let visits = if left_length == right_length {
            left_length as u64
        } else {
            0
        };
        self.charge(visits)?;
        Ok(self.heap.get(left).map_err(kernel_heap_error::map)?
            == self.heap.get(right).map_err(kernel_heap_error::map)?)
    }
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

impl fmt::Debug for KernelHeapContext<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KernelHeapContext")
            .field("usage", &self.usage)
            .field("heap_usage", &self.heap.usage())
            .finish_non_exhaustive()
    }
}
