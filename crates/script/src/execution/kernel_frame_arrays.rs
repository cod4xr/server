// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_frame::KernelFrame;
use crate::execution::{
    KernelError, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
};

impl KernelFrame {
    pub(super) fn stack_or_local(
        &mut self,
        instruction: KernelInstruction,
        frame: usize,
        mut heap: Option<&mut KernelHeapContext<'_>>,
        pc: usize,
    ) -> Result<(), KernelError> {
        match instruction {
            KernelInstruction::LoadLocal(slot) => {
                let value = copy(self.locals.load(slot, pc)?, frame, heap.as_deref_mut())
                    .map_err(|kind| kernel_error::at(pc, kind))?;
                self.stack.push(value, pc)
            }
            KernelInstruction::Duplicate => {
                let value = self
                    .stack
                    .top()
                    .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
                let value = copy(value, frame, heap.as_deref_mut())
                    .map_err(|kind| kernel_error::at(pc, kind))?;
                self.stack.push(value, pc)
            }
            KernelInstruction::StoreLocal(slot) => {
                // Validate both slots before releasing a replaced managed owner.
                let value = self
                    .stack
                    .top()
                    .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
                if let Some(old) = self.locals.peek(slot, pc)? {
                    release(old, frame, heap.as_deref_mut())
                        .map_err(|kind| kernel_error::at(pc, kind))?;
                }
                self.stack.pop(pc)?;
                self.locals.store(slot, value, pc)
            }
            KernelInstruction::Discard => {
                let value = self.stack.pop(pc)?;
                release(value, frame, heap).map_err(|kind| kernel_error::at(pc, kind))
            }
            _ => Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState)),
        }
    }

    pub(super) fn bind_array_arguments(
        &mut self,
        arguments: &[KernelValue],
        mut heap: Option<&mut KernelHeapContext<'_>>,
    ) -> Result<(), KernelError> {
        for (slot, value) in arguments.iter().copied().enumerate() {
            if let KernelValue::Array(id) = value {
                let context = heap.as_deref_mut().ok_or_else(|| {
                    kernel_error::general(KernelErrorKind::ObjectExecutionUnsupported)
                })?;
                let id = context
                    .copy_host_array(0, id)
                    .map_err(kernel_error::general)?;
                self.locals.store(slot, KernelValue::Array(id), 0)?;
            }
        }
        Ok(())
    }
}

fn copy(
    value: KernelValue,
    frame: usize,
    heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<KernelValue, KernelErrorKind> {
    match value {
        KernelValue::Array(id) => heap
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .copy_owned_array(frame, id)
            .map(KernelValue::Array),
        value => Ok(value),
    }
}

pub(super) fn release(
    value: KernelValue,
    frame: usize,
    heap: Option<&mut KernelHeapContext<'_>>,
) -> Result<(), KernelErrorKind> {
    if let KernelValue::Array(id) = value {
        heap.ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .release_owned_array(frame, id)?;
    }
    Ok(())
}
