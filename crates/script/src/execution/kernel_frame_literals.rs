// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_frame::KernelFrame;
use crate::execution::{
    KernelError, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
    VerifiedKernelProgram,
};

enum KernelLiteralKind {
    Plain,
    Localized,
}

impl KernelFrame {
    pub(super) fn load_literal(
        &mut self,
        program: &VerifiedKernelProgram,
        instruction: KernelInstruction,
        heap: Option<&mut KernelHeapContext<'_>>,
        parents: impl Iterator<Item = KernelValue> + Clone,
        pc: usize,
    ) -> Result<(), KernelError> {
        let (index, kind) = match instruction {
            KernelInstruction::StringLiteral(index) => (index, KernelLiteralKind::Plain),
            KernelInstruction::LocalizedStringLiteral(index) => {
                (index, KernelLiteralKind::Localized)
            }
            _ => return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState)),
        };
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
        let value = match kind {
            KernelLiteralKind::Plain => KernelValue::String(value),
            KernelLiteralKind::Localized => KernelValue::LocalizedString(value),
        };
        self.stack.push(value, pc)
    }
}
