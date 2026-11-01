// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::KernelHeapStep;
use crate::execution::heap::kernel_array_operations;
use crate::execution::heap::{
    kernel_object_comparison, kernel_object_validation, kernel_string_operations,
};
use crate::execution::kernel_builtin;
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{
    KernelBuiltin, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
    VerifiedKernelProgram,
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
            .any(|value| matches!(value, KernelValue::Object(_) | KernelValue::Array(_)))
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
    program: &VerifiedKernelProgram,
    instruction: KernelInstruction,
    stack: &KernelStack,
    mut context: Option<&mut KernelHeapContext<'_>>,
    roots: impl Iterator<Item = KernelValue> + Clone,
) -> Result<KernelHeapStep, KernelErrorKind> {
    if let Some(value) = kernel_array_operations::apply(instruction, stack, context.as_deref_mut())?
    {
        return Ok(KernelHeapStep::Applied(Some(value)));
    }
    if let (KernelInstruction::Compare(comparison), Some((left, right))) =
        (instruction, stack.binary_values())
        && (matches!(left, KernelValue::Object(_)) || matches!(right, KernelValue::Object(_)))
    {
        return kernel_object_comparison::compare(context, comparison, left, right)
            .map(|value| KernelHeapStep::Applied(Some(value)));
    }
    if let KernelInstruction::LoadField(name) | KernelInstruction::StoreField(name) = instruction {
        let key = program
            .field_names
            .get(name)
            .ok_or(KernelErrorKind::InvalidFieldName { name: name as u64 })?;
        let Some(KernelValue::Object(id)) = stack.top() else {
            return Err(KernelErrorKind::ExpectedFieldObject);
        };
        let context = context.ok_or(KernelErrorKind::ObjectExecutionUnsupported)?;
        return if matches!(instruction, KernelInstruction::LoadField(_)) {
            context
                .field(id, key)
                .map(|value| KernelHeapStep::Applied(Some(value)))
        } else {
            let Some((value, _)) = stack.binary_values() else {
                return Err(KernelErrorKind::InvalidExecutionState);
            };
            context.store_field(id, key, value, roots)?;
            Ok(KernelHeapStep::Applied(None))
        };
    }
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
            .map(|value| KernelHeapStep::Applied(Some(value)));
    }
    if let KernelInstruction::Builtin {
        function,
        arguments,
    } = instruction
    {
        return kernel_builtin::evaluate(function, arguments, stack)
            .map(|value| KernelHeapStep::Applied(Some(value)));
    }
    if let (KernelInstruction::Size, Some(KernelValue::Object(id))) = (instruction, stack.top()) {
        let context = context.ok_or(KernelErrorKind::ObjectExecutionUnsupported)?;
        context.charge(1)?;
        kernel_object_validation::reference(context, id)?;
        return Ok(KernelHeapStep::Applied(Some(KernelValue::Number(
            LegacyNumber::Integer(1),
        ))));
    }
    kernel_string_operations::apply(instruction, stack, context, roots).map(|value| {
        value.map_or(KernelHeapStep::Unhandled, |value| {
            KernelHeapStep::Applied(Some(value))
        })
    })
}
