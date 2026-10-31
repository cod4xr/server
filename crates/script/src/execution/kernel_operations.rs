// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_locals::KernelLocals;
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{KernelError, KernelErrorKind, KernelInstruction, KernelValue};
use crate::legacy::{LegacyNumber, LegacyNumericPair};

pub(super) fn apply(
    instruction: KernelInstruction,
    stack: &mut KernelStack,
    locals: &mut KernelLocals,
    pc: usize,
) -> Result<(), KernelError> {
    let value = match instruction {
        KernelInstruction::Constant(value) => value,
        KernelInstruction::LoadLocal(slot) => locals.load(slot, pc)?,
        KernelInstruction::StoreLocal(slot) => {
            locals.store(slot, stack.pop(pc)?, pc)?;
            return Ok(());
        }
        KernelInstruction::Arithmetic(operator) => {
            let (left, right) = stack.pair(pc)?;
            KernelValue::Number(operator.evaluate(LegacyNumericPair::coerce(left, right)))
        }
        KernelInstruction::Compare(comparison) => {
            let (left, right) = stack.pair(pc)?;
            KernelValue::Number(LegacyNumericPair::coerce(left, right).compare(comparison))
        }
        KernelInstruction::Integer(operator) => {
            let (left, right) = stack.pair(pc)?;
            let value = operator
                .evaluate(left, right)
                .map_err(|error| kernel_error::at(pc, KernelErrorKind::Numeric(error)))?;
            KernelValue::Number(value)
        }
        KernelInstruction::Divide => {
            let (left, right) = stack.pair(pc)?;
            let value = LegacyNumericPair::coerce(left, right)
                .divide()
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
        KernelInstruction::Discard => {
            stack.pop(pc)?;
            return Ok(());
        }
        KernelInstruction::Jump(_)
        | KernelInstruction::Branch { .. }
        | KernelInstruction::Return => {
            return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState));
        }
    };
    stack.push(value, pc)
}
