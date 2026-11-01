// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_program;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::function_task_lowering;
use crate::compilation::{
    CompiledFunction, FunctionCompilationBudget, FunctionCompileError, FunctionSourceProfile,
};
use crate::declaration::LegacyFunctionSyntax;
use crate::execution::{KernelFunctionLayout, KernelInstruction, KernelValue};

/// Compiles the restricted zero-parameter, empty/single-scalar-return source profile.
///
/// Numeric/undefined literals, one-value groups, numeric operators, and lazy OR/AND
/// are lowered iteratively with original source spans. The result owns its proof/
/// map without retaining source/tree bytes. Caller module/name/developer binding
/// and activation remain separate. This is not whole-module or full GSC compilation.
///
/// # Errors
/// Rejects unsupported parameters/bodies/values, bad literal domains, malformed
/// private shapes, exhausted budgets, allocation failures, and kernel verification/
/// pure validation-execution failures. No partial callable result or external state
/// mutation escapes. Error locations preserve original bytes without source text.
pub fn compile_scalar_return_function(
    function: &LegacyFunctionSyntax<'_, '_>,
    budget: FunctionCompilationBudget,
) -> Result<CompiledFunction, FunctionCompileError> {
    let mut builder = FunctionBuilder::new(budget);
    let result = compile(&mut builder, function);
    result.map_err(|mut error| {
        error.work_steps = builder.ledger.usage.compiler_steps;
        error.peak_metadata_bytes = builder.ledger.usage.peak_metadata_bytes;
        error
    })
}

fn compile<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    function: &LegacyFunctionSyntax<'tree, 'source>,
) -> Result<CompiledFunction, FunctionCompileError> {
    builder.ledger.charge(1, function.span())?;
    let body = function_body::lower(function)?;
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::Return,
            span: body.return_span.clone(),
        },
        body.return_span.clone(),
    )?;
    if let Some(expression) = body.expression {
        builder.push_task(FunctionTask::Visit(expression), expression.span())?;
    } else {
        builder.push_task(
            FunctionTask::Emit {
                instruction: KernelInstruction::Constant(KernelValue::Undefined),
                span: body.return_span.clone(),
            },
            body.return_span,
        )?;
    }
    function_task_lowering::drain(builder)?;
    function_program::finish(
        builder,
        KernelFunctionLayout::default(),
        function.body().span(),
        FunctionSourceProfile::PureScalarReturn,
    )
}
