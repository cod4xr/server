// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_local_bindings::{ScalarLocalBindings, ScalarLocalDeclaration};
use crate::compilation::scalar_local_names;
use crate::compilation::scalar_local_statements;
use crate::compilation::scalar_program;
use crate::compilation::scalar_task::ScalarTask;
use crate::compilation::scalar_task_lowering;
use crate::compilation::{
    CompiledScalarFunction, ScalarCompilationBudget, ScalarCompileError, ScalarCompileErrorKind,
    ScalarSourceProfile,
};
use crate::declaration::LegacyFunctionSyntax;
use crate::execution::KernelFunctionLayout;
use crate::statement::{LegacyStatementKind, LegacyStatementListSyntax, LegacyStatementSyntax};

/// Compiles the explicit scalar-local statement profile with structural branch joins.
///
/// Accepts parameters, ordinary/compound assignments and integer-only postfix
/// statements for plain names, empty statements, blocks, conditionals, while and returns.
/// Fallthrough returns Undefined. Names use legacy identity;
/// Reads require initialization on every reaching structural path. Duplicate
/// parameters, unreachable statements and nonfinal immediate returns fail.
/// Other statements/targets/values remain unsupported, including skipped source.
/// Caller module/developer binding and native argument adaptation remain separate.
/// Code/maps own no source/names and publish after verification without executing
/// fabricated arguments. Numeric/type errors therefore occur at invocation.
///
/// # Errors
/// Returns located unsupported/binding/initialization/identifier failures, separately
/// admitted body/literal/kernel errors, exhausted compiler quotas and allocation/size
/// failures. No partial function, module activation or external mutation escapes.
pub fn compile_local_scalar_function(
    function: &LegacyFunctionSyntax<'_, '_>,
    budget: ScalarCompilationBudget,
) -> Result<CompiledScalarFunction, ScalarCompileError> {
    let mut builder = ScalarBuilder::new(budget);
    builder.locals = Some(ScalarLocalBindings::new());
    let result = compile(&mut builder, function);
    result.map_err(|mut error| {
        error.work_steps = builder.ledger.usage.compiler_steps;
        error.peak_metadata_bytes = builder.ledger.usage.peak_metadata_bytes;
        error
    })
}

fn compile<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    function: &LegacyFunctionSyntax<'tree, 'source>,
) -> Result<CompiledScalarFunction, ScalarCompileError> {
    builder.ledger.charge(1, function.span())?;
    for parameter in function.parameters().iter().copied() {
        builder.ledger.charge(1, parameter.span())?;
        scalar_local_names::declare(builder, parameter, ScalarLocalDeclaration::Parameter)?;
    }
    let body = LegacyStatementListSyntax::lower(function.body(), builder.ledger.budget.body)
        .map_err(|error| {
            scalar_compile_error::at(
                function.body().span(),
                ScalarCompileErrorKind::BodyList(error),
            )
        })?;
    let has_return = body
        .statements()
        .next_back()
        .map(LegacyStatementSyntax::from_node)
        .transpose()
        .map_err(|_| {
            scalar_compile_error::at(body.span(), ScalarCompileErrorKind::InvalidStructure)
        })?
        .is_some_and(|view| matches!(view.kind(), LegacyStatementKind::Return { .. }));
    if !has_return {
        scalar_local_statements::queue_return(builder, None, function.body().span())?;
    }
    for (index, syntax) in body.statements().rev().enumerate() {
        builder.ledger.charge(1, syntax.span())?;
        builder.push_task(
            ScalarTask::Statement {
                syntax,
                last: index == 0,
            },
            syntax.span(),
        )?;
    }
    drop(body);
    scalar_task_lowering::drain(builder)?;
    let count = builder.ledger.usage.local_bindings;
    let layout =
        KernelFunctionLayout::new(function.parameters().len() as u64, count).map_err(|error| {
            scalar_compile_error::at(function.span(), ScalarCompileErrorKind::Kernel(error))
        })?;
    scalar_program::finish(
        builder,
        layout,
        function.body().span(),
        ScalarSourceProfile::LocalStatements,
    )
}
