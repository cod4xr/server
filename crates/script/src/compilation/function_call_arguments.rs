// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{
    FunctionCompileError, FunctionCompileErrorKind, ModuleCompilationResource,
};
use crate::expression::ExpressionListCursor;
use crate::syntax::LegacySyntaxNode;

pub(super) fn count(
    builder: &mut FunctionBuilder<'_, '_>,
    arguments: LegacySyntaxNode<'_, '_>,
) -> Result<usize, FunctionCompileError> {
    let mut cursor = ExpressionListCursor::new(arguments);
    let mut count = 0_usize;
    while cursor.pending() {
        builder.ledger.charge(1, arguments.span())?;
        if let Some(expression) = cursor.advance().map_err(|_| invalid(arguments.span()))? {
            count = count.checked_add(1).ok_or_else(|| {
                function_compile_error::at(
                    expression.span(),
                    FunctionCompileErrorKind::SizeOverflow,
                )
            })?;
            let maximum = builder.ledger.budget.max_call_arguments;
            if count as u64 > maximum {
                return Err(function_compile_error::at(
                    expression.span(),
                    FunctionCompileErrorKind::ModuleBudgetExceeded {
                        resource: ModuleCompilationResource::CallArguments,
                        requested: count as u64,
                        maximum,
                    },
                ));
            }
        }
    }
    Ok(count)
}

pub(super) fn next<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    mut cursor: ExpressionListCursor<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let expression = cursor.advance().map_err(|_| invalid(span.clone()))?;
    if cursor.pending() {
        builder.push_task(
            FunctionTask::CallArguments {
                cursor,
                span: span.clone(),
            },
            span,
        )?;
    }
    if let Some(expression) = expression {
        builder.push_task(FunctionTask::Visit(expression), expression.span())?;
    }
    Ok(())
}

fn invalid(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::InvalidStructure)
}
