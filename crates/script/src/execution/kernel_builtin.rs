// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_stack::KernelStack;
use crate::execution::{KernelError, KernelErrorKind, KernelValue};
use crate::legacy::LegacyNumber;

/// Sealed deterministic builtins, without host callbacks, plugins or ambient authority.
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
    /// Tests primitives/checked ordinary structs for definedness. Requires a first
    /// argument; evaluated extras are discarded. Entity/thread lifetimes stay unsupported.
    IsDefined,
    /// Allocates an empty shared struct through an explicit object context.
    /// Supplied arguments are evaluated and discarded; no arity check is imposed.
    SpawnStruct,
}

pub(super) fn apply(
    function: KernelBuiltin,
    arguments: usize,
    stack: &mut KernelStack,
    pc: usize,
) -> Result<KernelValue, KernelError> {
    let accepted = match function {
        KernelBuiltin::IsInt
        | KernelBuiltin::IsFloat
        | KernelBuiltin::IsVector
        | KernelBuiltin::IsString => arguments == 1,
        KernelBuiltin::IsDefined => arguments >= 1,
        KernelBuiltin::SpawnStruct => {
            return Err(kernel_error::at(
                pc,
                KernelErrorKind::ObjectExecutionUnsupported,
            ));
        }
    };
    if !accepted {
        return Err(kernel_error::at(
            pc,
            KernelErrorKind::ArgumentCount {
                expected: 1,
                actual: arguments as u64,
            },
        ));
    }
    let value = stack.pop(pc)?;
    // Every discarded value required an earlier admitted push instruction. This
    // work is bounded by the verified stack and the invocation's global fuel.
    for _ in 1..arguments {
        stack.pop(pc)?;
    }
    let result = match function {
        KernelBuiltin::IsInt => matches!(value, KernelValue::Number(LegacyNumber::Integer(_))),
        KernelBuiltin::IsFloat => matches!(value, KernelValue::Number(LegacyNumber::Float(_))),
        KernelBuiltin::IsVector => matches!(value, KernelValue::Vector(_)),
        KernelBuiltin::IsString => matches!(value, KernelValue::String(_)),
        KernelBuiltin::SpawnStruct => {
            return Err(kernel_error::at(
                pc,
                KernelErrorKind::ObjectExecutionUnsupported,
            ));
        }
        KernelBuiltin::IsDefined => match value {
            KernelValue::Undefined => false,
            KernelValue::Number(_)
            | KernelValue::Vector(_)
            | KernelValue::Function(_)
            | KernelValue::String(_)
            | KernelValue::LocalizedString(_)
            | KernelValue::Object(_) => true,
        },
    };
    Ok(KernelValue::Number(LegacyNumber::Integer(i32::from(
        result,
    ))))
}
