// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_task::ScalarTask;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::statement::LegacyStatementListSyntax;
use crate::syntax::LegacySyntaxNode;

pub(super) fn queue<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let body =
        LegacyStatementListSyntax::lower(syntax, builder.ledger.budget.body).map_err(|error| {
            scalar_compile_error::at(syntax.span(), ScalarCompileErrorKind::BodyList(error))
        })?;
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
    Ok(())
}
