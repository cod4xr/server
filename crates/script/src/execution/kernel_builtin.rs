// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{KernelError, KernelErrorKind, KernelValue};
use crate::legacy::LegacyNumber;

/// Sealed pure builtin subset, without host callbacks, plugins or ambient authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelBuiltin {
    /// Original isint: true only for an integer value, including zero.
    IsInt,
    /// Original isfloat: true only for a float value, including nonfinite data.
    IsFloat,
    /// Original isvector: true only for an inline vector value.
    IsVector,
    /// Original isstring: true only for plain string values, not localized strings.
    IsString,
}

pub(super) fn apply(
    function: KernelBuiltin,
    arguments: usize,
    stack: &mut KernelStack,
    pc: usize,
) -> Result<KernelValue, KernelError> {
    if arguments != 1 {
        return Err(kernel_error::at(
            pc,
            KernelErrorKind::ArgumentCount {
                expected: 1,
                actual: arguments as u64,
            },
        ));
    }
    let value = stack.pop(pc)?;
    let result = match function {
        KernelBuiltin::IsInt => matches!(value, KernelValue::Number(LegacyNumber::Integer(_))),
        KernelBuiltin::IsFloat => matches!(value, KernelValue::Number(LegacyNumber::Float(_))),
        KernelBuiltin::IsVector => matches!(value, KernelValue::Vector(_)),
        KernelBuiltin::IsString => matches!(value, KernelValue::String(_)),
    };
    Ok(KernelValue::Number(LegacyNumber::Integer(i32::from(
        result,
    ))))
}
