// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelFunctionDefinition, KernelFunctionLayout,
    KernelInstruction, KernelValue,
};

pub(super) fn verify(
    instruction: KernelInstruction,
    pc: usize,
    count: usize,
    layout: KernelFunctionLayout,
    functions: Option<&[KernelFunctionDefinition<'_>]>,
    literals: usize,
    field_names: usize,
) -> Result<(), KernelError> {
    match instruction {
        KernelInstruction::Constant(KernelValue::Array(_)) => {
            Err(kernel_error::at(pc, KernelErrorKind::RuntimeArrayConstant))
        }
        KernelInstruction::LoadField(name) | KernelInstruction::StoreField(name)
            if name >= field_names =>
        {
            Err(kernel_error::at(
                pc,
                KernelErrorKind::InvalidFieldName { name: name as u64 },
            ))
        }
        KernelInstruction::Constant(KernelValue::Object(_)) => {
            Err(kernel_error::at(pc, KernelErrorKind::RuntimeObjectConstant))
        }
        KernelInstruction::StringLiteral(literal)
        | KernelInstruction::LocalizedStringLiteral(literal)
            if literal >= literals =>
        {
            Err(kernel_error::at(
                pc,
                KernelErrorKind::InvalidStringLiteral {
                    literal: literal as u64,
                },
            ))
        }
        KernelInstruction::Constant(KernelValue::String(_) | KernelValue::LocalizedString(_)) => {
            Err(kernel_error::at(pc, KernelErrorKind::RuntimeStringConstant))
        }
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
            target(functions, function, pc, KernelErrorKind::UnlinkedCall)
        }
        KernelInstruction::FunctionReference(function) => target(
            functions,
            function,
            pc,
            KernelErrorKind::UnlinkedFunctionReference,
        ),
        KernelInstruction::CallIndirect { .. } => {
            if functions.is_none() {
                return Err(kernel_error::at(pc, KernelErrorKind::UnlinkedCall));
            }
            instruction
                .stack_effect()
                .map_err(|kind| kernel_error::at(pc, kind))?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn target(
    functions: Option<&[KernelFunctionDefinition<'_>]>,
    function: usize,
    pc: usize,
    unlinked: KernelErrorKind,
) -> Result<(), KernelError> {
    let functions = functions.ok_or_else(|| kernel_error::at(pc, unlinked))?;
    if functions.get(function).is_none() {
        return Err(kernel_error::at(
            pc,
            KernelErrorKind::InvalidFunction {
                function: function as u64,
            },
        ));
    }
    Ok(())
}
