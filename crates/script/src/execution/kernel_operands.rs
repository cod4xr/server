// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelFunctionDefinition, KernelFunctionLayout, KernelInstruction,
};

pub(super) fn verify(
    instruction: KernelInstruction,
    pc: usize,
    count: usize,
    layout: KernelFunctionLayout,
    functions: Option<&[KernelFunctionDefinition<'_>]>,
) -> Result<(), KernelError> {
    match instruction {
        KernelInstruction::Jump(target) | KernelInstruction::Branch { target, .. }
            if target >= count =>
        {
            Err(kernel_error::at(
                pc,
                KernelErrorKind::InvalidTarget {
                    target: target as u64,
                },
            ))
        }
        KernelInstruction::LoadLocal(slot) | KernelInstruction::StoreLocal(slot)
            if slot >= layout.local_values =>
        {
            Err(kernel_error::at(
                pc,
                KernelErrorKind::InvalidLocal {
                    slot: slot as u64,
                    local_values: layout.local_values(),
                },
            ))
        }
        KernelInstruction::Call { function, .. } => {
            let functions =
                functions.ok_or_else(|| kernel_error::at(pc, KernelErrorKind::UnlinkedCall))?;
            functions.get(function).ok_or_else(|| {
                kernel_error::at(
                    pc,
                    KernelErrorKind::InvalidFunction {
                        function: function as u64,
                    },
                )
            })?;
            Ok(())
        }
        _ => Ok(()),
    }
}
