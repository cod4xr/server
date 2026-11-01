// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue};

pub(super) fn apply(
    instruction: KernelInstruction,
    stack: &KernelStack,
    context: Option<&mut KernelHeapContext<'_>>,
) -> Result<Option<KernelValue>, KernelErrorKind> {
    match (instruction, stack.top()) {
        (KernelInstruction::Index, Some(KernelValue::Array(value))) => {
            let (index, _) = stack
                .binary_values()
                .ok_or(KernelErrorKind::InvalidExecutionState)?;
            context
                .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
                .array_index(value, index)
                .map(Some)
        }
        (KernelInstruction::Size, Some(KernelValue::Array(value))) => context
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .array_size(value)
            .map(Some),
        _ => Ok(None),
    }
}
