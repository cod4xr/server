// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{LegacyFunctionVisibility, LegacyIdentifier};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_call_scope::FunctionCallTarget;
use crate::compilation::function_compile_error;
use crate::compilation::function_lowering_mode::FunctionLoweringMode;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::{KernelBuiltin, KernelInstruction};
use crate::expression::LegacyFunctionNameSyntax;

pub(super) enum FunctionNamedCallTarget {
    Script(usize),
    Builtin(KernelBuiltin),
}

impl FunctionNamedCallTarget {
    pub(super) fn instruction(self, arguments: usize) -> KernelInstruction {
        match self {
            Self::Script(function) => KernelInstruction::Call {
                function,
                arguments,
            },
            Self::Builtin(function) => KernelInstruction::Builtin {
                function,
                arguments,
            },
        }
    }
}

pub(super) fn named_call(
    builder: &mut FunctionBuilder<'_, '_>,
    callee: LegacyFunctionNameSyntax<'_, '_>,
) -> Result<FunctionNamedCallTarget, FunctionCompileError> {
    if let Some(target) = find(builder, callee)? {
        return Ok(FunctionNamedCallTarget::Script(target.function));
    }
    let name = callee.name().as_bytes();
    let builtin = if name.eq_ignore_ascii_case(b"isint") {
        KernelBuiltin::IsInt
    } else if name.eq_ignore_ascii_case(b"isfloat") {
        KernelBuiltin::IsFloat
    } else if name.eq_ignore_ascii_case(b"isvector") {
        KernelBuiltin::IsVector
    } else if name.eq_ignore_ascii_case(b"isstring") {
        KernelBuiltin::IsString
    } else {
        return Err(unknown(callee));
    };
    Ok(FunctionNamedCallTarget::Builtin(builtin))
}

pub(super) fn resolve(
    builder: &mut FunctionBuilder<'_, '_>,
    callee: LegacyFunctionNameSyntax<'_, '_>,
) -> Result<usize, FunctionCompileError> {
    find(builder, callee)?
        .map(|target| target.function)
        .ok_or_else(|| unknown(callee))
}

fn unknown(callee: LegacyFunctionNameSyntax<'_, '_>) -> FunctionCompileError {
    function_compile_error::at(
        callee.name().span(),
        FunctionCompileErrorKind::UnknownFunction,
    )
}

fn find(
    builder: &mut FunctionBuilder<'_, '_>,
    callee: LegacyFunctionNameSyntax<'_, '_>,
) -> Result<Option<FunctionCallTarget>, FunctionCompileError> {
    if callee.module().is_some() {
        return Err(function_compile_error::at(
            callee.span(),
            FunctionCompileErrorKind::UnsupportedCall,
        ));
    }
    let token = callee.name();
    builder
        .ledger
        .identifier(token.as_bytes().len() as u64, token.span())?;
    let name = LegacyIdentifier::from_lexeme(token).map_err(|cause| {
        function_compile_error::at(token.span(), FunctionCompileErrorKind::Identifier(cause))
    })?;
    let FunctionLoweringMode::BoundModule { scope, caller } = builder.mode else {
        return Err(function_compile_error::at(
            callee.span(),
            FunctionCompileErrorKind::InvalidStructure,
        ));
    };
    let Some(target) = scope.find(name) else {
        return Ok(None);
    };
    if caller == LegacyFunctionVisibility::Ordinary
        && target.visibility == LegacyFunctionVisibility::DeveloperOnly
    {
        return Err(function_compile_error::at(
            token.span(),
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ));
    }
    Ok(Some(target))
}
