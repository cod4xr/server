// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{
    KernelErrorKind, KernelHeapContext, KernelInstruction, KernelSwitchComparison, KernelSwitchKey,
    KernelValue, VerifiedKernelProgram,
};
use crate::legacy::LegacyNumber;
use std::cmp::Ordering;

pub(super) fn apply(
    program: &VerifiedKernelProgram,
    instruction: KernelInstruction,
    stack: &KernelStack,
    context: Option<&mut KernelHeapContext<'_>>,
) -> Result<Option<KernelValue>, KernelErrorKind> {
    if !matches!(
        instruction,
        KernelInstruction::RequireSwitchKey | KernelInstruction::SwitchCompare { .. }
    ) {
        return Ok(None);
    }
    let value = stack.top().ok_or(KernelErrorKind::InvalidExecutionState)?;
    match instruction {
        KernelInstruction::RequireSwitchKey => {
            match value {
                KernelValue::Number(LegacyNumber::Integer(_)) => {}
                KernelValue::String(id) => context
                    .ok_or(KernelErrorKind::HeapContextRequired)?
                    .require_switch_string(id)?,
                _ => return Err(KernelErrorKind::ExpectedSwitchKey),
            }
            Ok(Some(value))
        }
        KernelInstruction::SwitchCompare { key, comparison } => {
            let order = compare(program, key, value, context)?;
            let result = match comparison {
                KernelSwitchComparison::Equal => order == Ordering::Equal,
                KernelSwitchComparison::Less => order == Ordering::Less,
            };
            Ok(Some(KernelValue::Number(LegacyNumber::Integer(i32::from(
                result,
            )))))
        }
        _ => Err(KernelErrorKind::InvalidExecutionState),
    }
}

fn compare(
    program: &VerifiedKernelProgram,
    key: KernelSwitchKey,
    value: KernelValue,
    context: Option<&mut KernelHeapContext<'_>>,
) -> Result<Ordering, KernelErrorKind> {
    let label = match key {
        KernelSwitchKey::StringLabel(index) => Some(
            program
                .switch_labels
                .get(index)
                .ok_or(KernelErrorKind::InvalidSwitchLabel {
                    label: index as u64,
                })?
                .as_slice(),
        ),
        KernelSwitchKey::Integer(_) => None,
    };
    match value {
        KernelValue::Number(LegacyNumber::Integer(value)) => Ok(match key {
            KernelSwitchKey::Integer(case) => value.cmp(&case),
            KernelSwitchKey::StringLabel(_) => Ordering::Less,
        }),
        KernelValue::String(id) => {
            let context = context.ok_or(KernelErrorKind::HeapContextRequired)?;
            if let Some(label) = label {
                context.compare_switch_string(id, label)
            } else {
                context.require_switch_string(id)?;
                Ok(Ordering::Greater)
            }
        }
        _ => Err(KernelErrorKind::ExpectedSwitchKey),
    }
}
