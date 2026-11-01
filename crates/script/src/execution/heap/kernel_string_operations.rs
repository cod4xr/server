// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue};
use crate::heap::StringId;
use crate::legacy::{LegacyArithmetic, LegacyComparison, LegacyNumber};

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
    if matches!(instruction, KernelInstruction::Size) {
        return match stack.top() {
            Some(KernelValue::String(id)) => context
                .ok_or(KernelErrorKind::HeapContextRequired)?
                .size(id)
                .map(Some),
            _ => Err(KernelErrorKind::ExpectedSized),
        };
    }
    if matches!(instruction, KernelInstruction::Index) {
        return match stack.binary_values() {
            Some((index, KernelValue::String(id))) => context
                .ok_or(KernelErrorKind::HeapContextRequired)?
                .index(id, index, roots)
                .map(Some),
            _ => Ok(None),
        };
    }
    if !matches!(
        instruction,
        KernelInstruction::Arithmetic(_)
            | KernelInstruction::Divide
            | KernelInstruction::Compare(_)
    ) {
        return Ok(None);
    }
    let Some((left, right)) = stack.binary_values() else {
        return Ok(None);
    };
    if left.string_id().is_none() && right.string_id().is_none() {
        return Ok(None);
    }
    if let (KernelValue::LocalizedString(left), KernelValue::LocalizedString(right)) = (left, right)
    {
        return match instruction {
            KernelInstruction::Compare(comparison) => compare(context, left, right, comparison),
            _ => Err(KernelErrorKind::UnmatchingTypes),
        };
    }
    let (KernelValue::String(left), KernelValue::String(right)) = (left, right) else {
        return Err(KernelErrorKind::UnmatchingTypes);
    };
    match instruction {
        KernelInstruction::Arithmetic(LegacyArithmetic::Add) => {
            let context = context.ok_or(KernelErrorKind::HeapContextRequired)?;
            context
                .concatenate(left, right, roots)
                .map(KernelValue::String)
                .map(Some)
        }
        KernelInstruction::Compare(
            comparison @ (LegacyComparison::Equal | LegacyComparison::NotEqual),
        ) => compare(context, left, right, comparison),
        _ => Err(KernelErrorKind::UnmatchingTypes),
    }
}

fn compare(
    context: Option<&mut KernelHeapContext<'_>>,
    left: StringId,
    right: StringId,
    comparison: LegacyComparison,
) -> Result<Option<KernelValue>, KernelErrorKind> {
    if !matches!(
        comparison,
        LegacyComparison::Equal | LegacyComparison::NotEqual
    ) {
        return Err(KernelErrorKind::UnmatchingTypes);
    }
    let context = context.ok_or(KernelErrorKind::HeapContextRequired)?;
    let equal = context.equal(left, right)?;
    let result = if comparison == LegacyComparison::Equal {
        equal
    } else {
        !equal
    };
    Ok(Some(KernelValue::Number(LegacyNumber::Integer(i32::from(
        result,
    )))))
}
