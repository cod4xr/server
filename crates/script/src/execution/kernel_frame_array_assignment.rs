// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::KernelArrayMutation;
use crate::execution::kernel_error;
use crate::execution::kernel_frame::KernelFrame;
use crate::execution::kernel_frame_arrays;
use crate::execution::{
    KernelError, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
};

impl KernelFrame {
    pub(super) fn array_assignment(
        &mut self,
        instruction: KernelInstruction,
        frame: usize,
        heap: Option<&mut KernelHeapContext<'_>>,
        parents: impl Iterator<Item = KernelValue> + Clone,
        pc: usize,
    ) -> Result<(), KernelError> {
        let (slot, index, operation, consumed) =
            match instruction {
                KernelInstruction::StoreArrayLocal(slot) => {
                    let (rhs, index) = self.stack.binary_values().ok_or_else(|| {
                        kernel_error::at(pc, KernelErrorKind::InvalidExecutionState)
                    })?;
                    (slot, index, KernelArrayMutation::Assign(rhs), 2)
                }
                KernelInstruction::ClearArrayLocal(slot) => {
                    let index = self.stack.top().ok_or_else(|| {
                        kernel_error::at(pc, KernelErrorKind::InvalidExecutionState)
                    })?;
                    (slot, index, KernelArrayMutation::Clear, 1)
                }
                _ => return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState)),
            };
        operation
            .validate()
            .map_err(|kind| kernel_error::at(pc, kind))?;
        let context =
            heap.ok_or_else(|| kernel_error::at(pc, KernelErrorKind::ObjectExecutionUnsupported))?;
        let value = match self.locals.load(slot, pc)? {
            KernelValue::Array(value) => value,
            KernelValue::Undefined if matches!(operation, KernelArrayMutation::Assign(_)) => {
                let value = context
                    .construct_empty_array(frame, self.values().chain(parents.clone()))
                    .map_err(|kind| kernel_error::at(pc, kind))?;
                self.locals.store(slot, KernelValue::Array(value), pc)?;
                value
            }
            KernelValue::String(_) => {
                return Err(kernel_error::at(pc, KernelErrorKind::ReadOnlyStringIndex));
            }
            KernelValue::Vector(_) => {
                return Err(kernel_error::at(pc, KernelErrorKind::ReadOnlyVectorIndex));
            }
            _ => return Err(kernel_error::at(pc, KernelErrorKind::ExpectedArrayTarget)),
        };
        context
            .mutate_owned_array(frame, value, index, operation, self.values().chain(parents))
            .map_err(|kind| kernel_error::at(pc, kind))?;
        for _ in 0..consumed {
            let value = self.stack.pop(pc)?;
            kernel_frame_arrays::release(value, frame, Some(&mut *context))
                .map_err(|kind| kernel_error::at(pc, kind))?;
        }
        Ok(())
    }
}
