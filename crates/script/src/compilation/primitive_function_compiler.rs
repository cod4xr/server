// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_local_bindings::FunctionLocalDeclaration;
use crate::compilation::function_local_names;
use crate::compilation::function_local_statements;
use crate::compilation::function_program;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::function_task_lowering;
use crate::compilation::{
    CompiledFunction, FunctionCompilationBudget, FunctionCompileError, FunctionCompileErrorKind,
    FunctionSourceProfile,
};
use crate::declaration::LegacyFunctionSyntax;
use crate::execution::KernelFunctionLayout;
use crate::statement::{LegacyStatementKind, LegacyStatementListSyntax, LegacyStatementSyntax};

/// Compiles the explicit primitive statement profile with structural branch joins.
///
/// Accepts parameters, ordinary/compound assignments and integer-only postfix
/// statements for plain names, empty statements, blocks, conditionals, while,
/// scoped break/continue, primitive for clauses and returns.
/// Numeric/vector expressions include singleton grouping and three-component
/// construction in explicit right-to-left component order. Types check at invocation.
/// Fallthrough returns Undefined. Names use legacy identity;
/// Reads require initialization on every reaching structural path. Duplicate
/// parameters, unreachable statements and nonfinal immediate returns fail.
/// For updates merge continuing paths; an unreachable update checks incoming facts.
/// Loop exits conservatively permit zero iterations. Clauses accept only primitive
/// assignment/update/empty syntax; this is not full native scope/constant-folding parity.
/// Other statements/targets/values remain unsupported, including skipped source.
/// Caller module/developer binding and native argument adaptation remain separate.
/// Code/maps own no source/names and publish after verification without executing
/// fabricated arguments. Numeric/type errors therefore occur at invocation.
///
/// # Errors
/// Returns located unsupported/binding/initialization/identifier failures, separately
/// admitted body/literal/kernel errors, exhausted compiler quotas and allocation/size
/// failures. No partial function, module activation or external mutation escapes.
pub fn compile_primitive_function(
    function: &LegacyFunctionSyntax<'_, '_>,
    budget: FunctionCompilationBudget,
) -> Result<CompiledFunction, FunctionCompileError> {
    let mut builder = FunctionBuilder::new(budget, FunctionSourceProfile::PrimitiveStatements);
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
    for parameter in function.parameters().iter().copied() {
        builder.ledger.charge(1, parameter.span())?;
        function_local_names::declare(builder, parameter, FunctionLocalDeclaration::Parameter)?;
    }
    let body = LegacyStatementListSyntax::lower(function.body(), builder.ledger.budget.body)
        .map_err(|error| {
            function_compile_error::at(
                function.body().span(),
                FunctionCompileErrorKind::BodyList(error),
            )
        })?;
    let has_return = body
        .statements()
        .next_back()
        .map(LegacyStatementSyntax::from_node)
        .transpose()
        .map_err(|_| {
            function_compile_error::at(body.span(), FunctionCompileErrorKind::InvalidStructure)
        })?
        .is_some_and(|view| matches!(view.kind(), LegacyStatementKind::Return { .. }));
    if !has_return {
        function_local_statements::queue_return(builder, None, function.body().span())?;
    }
    for (index, syntax) in body.statements().rev().enumerate() {
        builder.ledger.charge(1, syntax.span())?;
        builder.push_task(
            FunctionTask::Statement {
                syntax,
                last: index == 0,
            },
            syntax.span(),
        )?;
    }
    drop(body);
    function_task_lowering::drain(builder)?;
    let count = builder.ledger.usage.local_bindings;
    let layout =
        KernelFunctionLayout::new(function.parameters().len() as u64, count).map_err(|error| {
            function_compile_error::at(function.span(), FunctionCompileErrorKind::Kernel(error))
        })?;
    function_program::finish(builder, layout, function.body().span())
}
