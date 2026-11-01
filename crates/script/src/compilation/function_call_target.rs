// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{LegacyFunctionVisibility, LegacyIdentifier};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_lowering_mode::FunctionLoweringMode;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::expression::LegacyFunctionNameSyntax;

pub(super) fn resolve(
    builder: &mut FunctionBuilder<'_, '_>,
    callee: LegacyFunctionNameSyntax<'_, '_>,
) -> Result<usize, FunctionCompileError> {
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
    let target = scope.find(name).ok_or_else(|| {
        function_compile_error::at(token.span(), FunctionCompileErrorKind::UnknownFunction)
    })?;
    if caller == LegacyFunctionVisibility::Ordinary
        && target.visibility == LegacyFunctionVisibility::DeveloperOnly
    {
        return Err(function_compile_error::at(
            token.span(),
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ));
    }
    Ok(target.function)
}
