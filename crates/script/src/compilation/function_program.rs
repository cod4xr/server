// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::{
    CompiledFunction, FunctionCompileError, FunctionCompileErrorKind, FunctionSourceProfile,
};
use crate::execution::{KernelError, KernelFunctionLayout, VerifiedKernelProgram};

pub(super) fn finish(
    builder: &mut FunctionBuilder<'_, '_>,
    layout: KernelFunctionLayout,
    fallback: Range<u64>,
) -> Result<CompiledFunction, FunctionCompileError> {
    let mode = builder.mode.standalone().ok_or_else(|| {
        function_compile_error::at(fallback.clone(), FunctionCompileErrorKind::InvalidStructure)
    })?;
    let (code, spans) = builder.resolve(fallback.clone())?;
    let program = VerifiedKernelProgram::verify_function(&code, layout, mode.verification)
        .map_err(|error| kernel_error(error, &spans, fallback.clone()))?;
    drop(code);
    if mode.profile == FunctionSourceProfile::PureScalarReturn {
        let validation = program
            .execute(mode.validation)
            .map_err(|error| kernel_error(error, &spans, fallback))?;
        builder.ledger.usage.validation_steps = Some(validation.steps);
    }
    builder.ledger.usage.verification_steps = program.usage().verification_steps;
    Ok(CompiledFunction {
        profile: mode.profile,
        program,
        spans,
        usage: builder.ledger.usage,
    })
}

fn kernel_error(
    error: KernelError,
    spans: &[Range<u64>],
    fallback: Range<u64>,
) -> FunctionCompileError {
    let span = error
        .instruction
        .and_then(|pc| usize::try_from(pc).ok())
        .and_then(|pc| spans.get(pc))
        .cloned()
        .unwrap_or(fallback);
    function_compile_error::at(span, FunctionCompileErrorKind::Kernel(error))
}
