// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{self, KernelHeapStep};
use crate::execution::kernel_error;
use crate::execution::kernel_frame::KernelFrame;
use crate::execution::kernel_frame_arrays;
use crate::execution::kernel_operations;
use crate::execution::{
    KernelError, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
    VerifiedKernelProgram,
};

impl KernelFrame {
    pub(super) fn data_step(
        &mut self,
        program: &VerifiedKernelProgram,
        instruction: KernelInstruction,
        owner: usize,
        mut heap: Option<&mut KernelHeapContext<'_>>,
        parents: impl Iterator<Item = KernelValue> + Clone,
        pc: usize,
    ) -> Result<(), KernelError> {
        if let KernelInstruction::LoadScriptRoot(root) = instruction {
            let value = heap
                .as_deref_mut()
                .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::ScriptRootContextRequired))?
                .script_root(root, owner)
                .map_err(|kind| kernel_error::at(pc, kind))?;
            return self.stack.push(value, pc);
        }
        let roots = self.values().chain(parents);
        let value = heap::apply(
            program,
            instruction,
            &self.stack,
            heap.as_deref_mut(),
            roots,
        )
        .map_err(|kind| kernel_error::at(pc, kind))?;
        if let KernelHeapStep::Applied(value) = value {
            let (consumed, produced) = instruction
                .stack_effect()
                .map_err(|kind| kernel_error::at(pc, kind))?;
            if produced != usize::from(value.is_some()) {
                return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState));
            }
            for _ in 0..consumed {
                let value = self.stack.pop(pc)?;
                kernel_frame_arrays::release(value, owner, heap.as_deref_mut())
                    .map_err(|kind| kernel_error::at(pc, kind))?;
            }
            if let Some(value) = value {
                self.stack.push(value, pc)?;
            }
            Ok(())
        } else {
            kernel_operations::apply(instruction, &mut self.stack, pc)
        }
    }
}
