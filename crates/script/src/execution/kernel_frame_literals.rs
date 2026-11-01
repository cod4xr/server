// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_frame::KernelFrame;
use crate::execution::{
    KernelError, KernelErrorKind, KernelHeapContext, KernelValue, VerifiedKernelProgram,
};

impl KernelFrame {
    pub(super) fn load_literal(
        &mut self,
        program: &VerifiedKernelProgram,
        index: usize,
        heap: Option<&mut KernelHeapContext<'_>>,
        parents: impl Iterator<Item = KernelValue> + Clone,
        pc: usize,
    ) -> Result<(), KernelError> {
        let heap =
            heap.ok_or_else(|| kernel_error::at(pc, KernelErrorKind::HeapContextRequired))?;
        let bytes = program
            .string_literals
            .get(index)
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
        let cache_index = program
            .literal_offset
            .checked_add(index)
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::SizeOverflow))?;
        let value = heap
            .literal(cache_index, bytes, self.values().chain(parents))
            .map_err(|kind| kernel_error::at(pc, kind))?;
        self.stack.push(KernelValue::String(value), pc)
    }
}
