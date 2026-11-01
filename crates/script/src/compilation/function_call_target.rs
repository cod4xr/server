// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{LegacyFunctionVisibility, LegacyIdentifier};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_call_scope::{FunctionCallOrigin, FunctionCallTarget};
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
    let target = find(builder, callee)?;
    if let Some(target) = target.filter(|target| target.origin == FunctionCallOrigin::Direct) {
        return script(builder, callee, target).map(FunctionNamedCallTarget::Script);
    }
    if callee.module().is_some() {
        return Err(unknown(callee));
    }
    if let Some(builtin) = builtin(callee.name().as_bytes()) {
        return Ok(FunctionNamedCallTarget::Builtin(builtin));
    }
    script(builder, callee, target.ok_or_else(|| unknown(callee))?)
        .map(FunctionNamedCallTarget::Script)
}

fn builtin(name: &[u8]) -> Option<KernelBuiltin> {
    let builtin = if name.eq_ignore_ascii_case(b"isint") {
        KernelBuiltin::IsInt
    } else if name.eq_ignore_ascii_case(b"isfloat") {
        KernelBuiltin::IsFloat
    } else if name.eq_ignore_ascii_case(b"isvector") {
        KernelBuiltin::IsVector
    } else if name.eq_ignore_ascii_case(b"isstring") {
        KernelBuiltin::IsString
    } else if name.eq_ignore_ascii_case(b"isdefined") {
        KernelBuiltin::IsDefined
    } else if name.eq_ignore_ascii_case(b"spawnstruct") {
        KernelBuiltin::SpawnStruct
    } else {
        return None;
    };
    Some(builtin)
}

pub(super) fn resolve(
    builder: &mut FunctionBuilder<'_, '_>,
    callee: LegacyFunctionNameSyntax<'_, '_>,
) -> Result<usize, FunctionCompileError> {
    let target = find(builder, callee)?.ok_or_else(|| unknown(callee))?;
    script(builder, callee, target)
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
    let FunctionLoweringMode::BoundModule {
        scope,
        caller: _,
        module,
    } = builder.mode
    else {
        return Err(function_compile_error::at(
            callee.span(),
            FunctionCompileErrorKind::InvalidStructure,
        ));
    };
    let module = if let Some(path) = callee.module() {
        let Some(modules) = scope.modules else {
            return Err(function_compile_error::at(
                callee.span(),
                FunctionCompileErrorKind::UnsupportedCall,
            ));
        };
        modules.resolve(path, &mut builder.ledger)?
    } else {
        module
    };
    let token = callee.name();
    builder
        .ledger
        .identifier(token.as_bytes().len() as u64, token.span())?;
    let name = LegacyIdentifier::from_lexeme(token).map_err(|cause| {
        function_compile_error::at(token.span(), FunctionCompileErrorKind::Identifier(cause))
    })?;
    Ok(scope
        .find(module, name)
        .filter(|target| callee.module().is_none() || target.origin == FunctionCallOrigin::Direct))
}

fn script(
    builder: &FunctionBuilder<'_, '_>,
    callee: LegacyFunctionNameSyntax<'_, '_>,
    target: FunctionCallTarget,
) -> Result<usize, FunctionCompileError> {
    let FunctionLoweringMode::BoundModule { caller, .. } = builder.mode else {
        return Err(function_compile_error::at(
            callee.span(),
            FunctionCompileErrorKind::InvalidStructure,
        ));
    };
    if caller == LegacyFunctionVisibility::Ordinary
        && target.visibility == LegacyFunctionVisibility::DeveloperOnly
    {
        return Err(function_compile_error::at(
            callee.name().span(),
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ));
    }
    Ok(target.function)
}
