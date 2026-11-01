// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue};
use crate::legacy::{LegacyArithmetic, LegacyComparison, LegacyNumber};

pub(in crate::execution) fn validate_arguments(
    arguments: &[KernelValue],
    context: Option<&mut KernelHeapContext<'_>>,
) -> Result<(), KernelErrorKind> {
    if let Some(context) = context {
        return context.validate(arguments);
    }
    if arguments
        .iter()
        .any(|value| matches!(value, KernelValue::String(_)))
    {
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
    if !matches!(left, KernelValue::String(_)) && !matches!(right, KernelValue::String(_)) {
        return Ok(None);
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
        ) => {
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
        _ => Err(KernelErrorKind::UnmatchingTypes),
    }
}
