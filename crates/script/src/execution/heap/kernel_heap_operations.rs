// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::{kernel_object_validation, kernel_string_operations};
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{
    KernelBuiltin, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
};
use crate::legacy::LegacyNumber;

pub(in crate::execution) fn validate_arguments(
    arguments: &[KernelValue],
    context: Option<&mut KernelHeapContext<'_>>,
) -> Result<(), KernelErrorKind> {
    if context
        .as_ref()
        .is_none_or(|context| context.objects.is_none())
        && arguments
            .iter()
            .any(|value| matches!(value, KernelValue::Object(_)))
    {
        return Err(KernelErrorKind::ObjectExecutionUnsupported);
    }
    if let Some(context) = context {
        return context.validate(arguments);
    }
    if arguments.iter().any(|value| value.string_id().is_some()) {
        return Err(KernelErrorKind::HeapContextRequired);
    }
    Ok(())
}

pub(in crate::execution) fn apply(
    instruction: KernelInstruction,
    stack: &KernelStack,
    context: Option<&mut KernelHeapContext<'_>>,
    roots: impl Iterator<Item = KernelValue> + Clone,
) -> Result<Option<KernelValue>, KernelErrorKind> {
    if matches!(
        instruction,
        KernelInstruction::Builtin {
            function: KernelBuiltin::SpawnStruct,
            ..
        }
    ) {
        return context
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .allocate_object(roots)
            .map(KernelValue::Object)
            .map(Some);
    }
    if let (KernelInstruction::Size, Some(KernelValue::Object(id))) = (instruction, stack.top()) {
        let context = context.ok_or(KernelErrorKind::ObjectExecutionUnsupported)?;
        context.charge(1)?;
        kernel_object_validation::reference(context, id)?;
        return Ok(Some(KernelValue::Number(LegacyNumber::Integer(1))));
    }
    kernel_string_operations::apply(instruction, stack, context, roots)
}
