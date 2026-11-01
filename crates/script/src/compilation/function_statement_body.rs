// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::statement::LegacyStatementListSyntax;
use crate::syntax::LegacySyntaxNode;

pub(super) fn queue<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let body =
        LegacyStatementListSyntax::lower(syntax, builder.ledger.budget.body).map_err(|error| {
            function_compile_error::at(syntax.span(), FunctionCompileErrorKind::BodyList(error))
        })?;
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
    Ok(())
}
