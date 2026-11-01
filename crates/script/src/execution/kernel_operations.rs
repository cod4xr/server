// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_builtin;
use crate::execution::kernel_comparison_values;
use crate::execution::kernel_error;
use crate::execution::kernel_index_values;
use crate::execution::kernel_integer_value;
use crate::execution::kernel_locals::KernelLocals;
use crate::execution::kernel_numeric_values;
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{KernelError, KernelErrorKind, KernelInstruction, KernelValue};
use crate::legacy::LegacyNumber;

pub(super) fn apply(
    instruction: KernelInstruction,
    stack: &mut KernelStack,
    locals: &mut KernelLocals,
    pc: usize,
) -> Result<(), KernelError> {
    let value = match instruction {
        KernelInstruction::Constant(value) => value,
        KernelInstruction::LoadLocal(slot) => locals.load(slot, pc)?,
        KernelInstruction::Builtin {
            function,
            arguments,
        } => kernel_builtin::apply(function, arguments, stack, pc)?,
        KernelInstruction::RequireInteger => kernel_integer_value::require(stack.pop(pc)?)
            .map_err(|kind| kernel_error::at(pc, kind))?,
        KernelInstruction::StoreLocal(slot) => {
            locals.store(slot, stack.pop(pc)?, pc)?;
            return Ok(());
        }
        KernelInstruction::Compare(comparison) => {
            let right = stack.pop(pc)?;
            let left = stack.pop(pc)?;
            kernel_comparison_values::compare(comparison, left, right)
                .map_err(|kind| kernel_error::at(pc, kind))?
        }
        KernelInstruction::Arithmetic(_) | KernelInstruction::Divide => {
            let right = stack.pop(pc)?;
            let left = stack.pop(pc)?;
            kernel_numeric_values::binary(instruction, left, right)
                .map_err(|kind| kernel_error::at(pc, kind))?
        }
        KernelInstruction::MakeVector => {
            let x = stack.pop(pc)?;
            let y = stack.pop(pc)?;
            let z = stack.pop(pc)?;
            kernel_numeric_values::construct([x, y, z])
                .map_err(|kind| kernel_error::at(pc, kind))?
        }
        KernelInstruction::Index => {
            let object = stack.pop(pc)?;
            let index = stack.pop(pc)?;
            kernel_index_values::read(object, index).map_err(|kind| kernel_error::at(pc, kind))?
        }
        KernelInstruction::Size => {
            return Err(kernel_error::at(pc, KernelErrorKind::ExpectedSized));
        }
        KernelInstruction::Integer(operator) => {
            let (left, right) = stack.pair(pc)?;
            let value = operator
                .evaluate(left, right)
                .map_err(|error| kernel_error::at(pc, KernelErrorKind::Numeric(error)))?;
            KernelValue::Number(value)
        }
        KernelInstruction::IntegerUpdate(update) => {
            let value = update
                .evaluate(stack.number(pc)?)
                .map_err(|error| kernel_error::at(pc, KernelErrorKind::Numeric(error)))?;
            KernelValue::Number(value)
        }
        KernelInstruction::CastBool => {
            let number = stack.number(pc)?;
            KernelValue::Number(LegacyNumber::Integer(i32::from(number.as_bool())))
        }
        KernelInstruction::LogicalNot => KernelValue::Number(stack.number(pc)?.logical_not()),
        KernelInstruction::BitwiseNot => {
            let value = stack
                .number(pc)?
                .bitwise_not()
                .map_err(|error| kernel_error::at(pc, KernelErrorKind::Numeric(error)))?;
            KernelValue::Number(value)
        }
        KernelInstruction::Duplicate => return stack.duplicate(pc),
        KernelInstruction::Swap => return stack.swap(pc),
        KernelInstruction::Discard => {
            stack.pop(pc)?;
            return Ok(());
        }
        KernelInstruction::Call { .. }
        | KernelInstruction::LoadField(_)
        | KernelInstruction::StoreField(_)
        | KernelInstruction::StringLiteral(_)
        | KernelInstruction::LocalizedStringLiteral(_)
        | KernelInstruction::CallIndirect { .. }
        | KernelInstruction::FunctionReference(_)
        | KernelInstruction::Jump(_)
        | KernelInstruction::Branch { .. }
        | KernelInstruction::Return => {
            return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState));
        }
    };
    stack.push(value, pc)
}
