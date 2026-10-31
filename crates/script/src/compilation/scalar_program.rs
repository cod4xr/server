// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::{
    CompiledScalarFunction, ScalarCompileError, ScalarCompileErrorKind, ScalarSourceProfile,
};
use crate::execution::{KernelError, KernelFunctionLayout, VerifiedKernelProgram};

pub(super) fn finish(
    builder: &mut ScalarBuilder<'_, '_>,
    layout: KernelFunctionLayout,
    fallback: Range<u64>,
    profile: ScalarSourceProfile,
) -> Result<CompiledScalarFunction, ScalarCompileError> {
    let (code, spans) = builder.resolve(fallback.clone())?;
    let program =
        VerifiedKernelProgram::verify_function(&code, layout, builder.ledger.budget.verification)
            .map_err(|error| kernel_error(error, &spans, fallback.clone()))?;
    drop(code);
    if profile == ScalarSourceProfile::PureReturn {
        let validation = program
            .execute(builder.ledger.budget.validation)
            .map_err(|error| kernel_error(error, &spans, fallback))?;
        builder.ledger.usage.validation_steps = Some(validation.steps);
    }
    builder.ledger.usage.verification_steps = program.usage().verification_steps;
    Ok(CompiledScalarFunction {
        profile,
        program,
        spans,
        usage: builder.ledger.usage,
    })
}

fn kernel_error(
    error: KernelError,
    spans: &[Range<u64>],
    fallback: Range<u64>,
) -> ScalarCompileError {
    let span = error
        .instruction
        .and_then(|pc| usize::try_from(pc).ok())
        .and_then(|pc| spans.get(pc))
        .cloned()
        .unwrap_or(fallback);
    scalar_compile_error::at(span, ScalarCompileErrorKind::Kernel(error))
}
