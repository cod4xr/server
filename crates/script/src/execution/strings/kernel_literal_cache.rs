// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::strings::kernel_heap_error;
use crate::execution::strings::kernel_string_allocation::KernelStringInput;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::StringId;

pub(super) struct KernelLiteralCache {
    pub(super) slots: Vec<Option<StringId>>,
    pub(super) active: Vec<usize>,
    pub(super) bytes: u64,
}

impl KernelLiteralCache {
    pub(super) fn empty() -> Self {
        Self {
            slots: Vec::new(),
            active: Vec::new(),
            bytes: 0,
        }
    }

    fn allocate(count: usize, bytes: u64) -> Result<Self, KernelErrorKind> {
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(count)
            .map_err(|_| KernelErrorKind::AllocationFailed)?;
        let mut active = Vec::new();
        active
            .try_reserve_exact(count)
            .map_err(|_| KernelErrorKind::AllocationFailed)?;
        slots.resize(count, None);
        Ok(Self {
            slots,
            active,
            bytes,
        })
    }

    pub(super) fn active_id(&self, index: usize) -> Result<StringId, KernelErrorKind> {
        let slot = self
            .active
            .get(index)
            .ok_or(KernelErrorKind::InvalidExecutionState)?;
        self.slots
            .get(*slot)
            .copied()
            .flatten()
            .ok_or(KernelErrorKind::InvalidExecutionState)
    }
}

impl KernelHeapContext<'_> {
    pub(in crate::execution) fn prepare_literals(
        &mut self,
        count: u64,
    ) -> Result<(), KernelErrorKind> {
        self.literals = KernelLiteralCache::empty();
        let bytes = count
            .checked_mul((size_of::<Option<StringId>>() + size_of::<usize>()) as u64)
            .ok_or(KernelErrorKind::SizeOverflow)?;
        self.scratch(bytes)?;
        self.charge(count)?;
        let count = usize::try_from(count).map_err(|_| KernelErrorKind::SizeOverflow)?;
        self.literals = KernelLiteralCache::allocate(count, bytes)?;
        Ok(())
    }

    pub(in crate::execution) fn literal(
        &mut self,
        index: usize,
        bytes: &[u8],
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<StringId, KernelErrorKind> {
        self.charge(2)?;
        if let Some(id) = self
            .literals
            .slots
            .get(index)
            .copied()
            .ok_or(KernelErrorKind::InvalidExecutionState)?
        {
            self.heap.get(id).map_err(kernel_heap_error::map)?;
            return Ok(id);
        }
        let id = self.allocate(KernelStringInput::Literal(bytes), roots)?;
        if self.literals.active.len() >= self.literals.slots.len() {
            return Err(KernelErrorKind::InvalidExecutionState);
        }
        *self
            .literals
            .slots
            .get_mut(index)
            .ok_or(KernelErrorKind::InvalidExecutionState)? = Some(id);
        self.literals.active.push(index);
        Ok(id)
    }
}
