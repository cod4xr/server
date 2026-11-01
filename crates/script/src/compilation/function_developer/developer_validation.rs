// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{LegacyIdentifier, LegacyScriptPath, ScriptPathBudget};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_lowering_mode::FunctionLoweringMode;
use crate::compilation::function_task::PendingFunctionKind;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::expression::LegacyFunctionNameSyntax;
use std::ops::Range;

pub(in crate::compilation) fn discarded(builder: &FunctionBuilder<'_, '_>) -> Option<usize> {
    builder
        .active_developer
        .as_ref()
        .filter(|frame| frame.discarded)
        .map(|frame| frame.start)
}

pub(in crate::compilation) fn emit_validation(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    if discarded(builder).is_none() {
        return Err(function_compile_error::at(
            span,
            FunctionCompileErrorKind::InvalidControlState,
        ));
    }
    builder
        .append(PendingFunctionKind::ValidationOnly, span)
        .map(|_| ())
}

pub(in crate::compilation) fn validate_name(
    builder: &mut FunctionBuilder<'_, '_>,
    callee: LegacyFunctionNameSyntax<'_, '_>,
) -> Result<(), FunctionCompileError> {
    let name = callee.name();
    builder
        .ledger
        .identifier(name.as_bytes().len() as u64, name.span())?;
    LegacyIdentifier::from_lexeme(name).map_err(|cause| {
        function_compile_error::at(name.span(), FunctionCompileErrorKind::Identifier(cause))
    })?;
    if let Some(path) = callee.module() {
        let maximum = match builder.mode {
            FunctionLoweringMode::BoundModule { scope, .. } => scope
                .modules
                .map_or(ScriptPathBudget::default(), |modules| modules.path_budget),
            FunctionLoweringMode::Standalone(_) => ScriptPathBudget::default(),
        };
        let visits = (path.as_bytes().len() as u64)
            .checked_mul(2)
            .ok_or_else(|| {
                function_compile_error::at(path.span(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        builder.ledger.identifier(visits, path.span())?;
        LegacyScriptPath::new(path.as_bytes(), maximum).map_err(|cause| {
            function_compile_error::at(
                path.span(),
                FunctionCompileErrorKind::ScriptPath(cause.kind),
            )
        })?;
    }
    Ok(())
}
