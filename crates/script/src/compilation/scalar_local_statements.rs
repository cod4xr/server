// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::scalar_body;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_local_assignment;
use crate::compilation::scalar_statement_body;
use crate::compilation::scalar_task::ScalarTask;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::{KernelInstruction, KernelValue};
use crate::statement::{LegacyStatementKind, LegacyStatementSyntax};
use crate::syntax::LegacySyntaxNode;

pub(super) fn lower<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
    last: bool,
) -> Result<(), ScalarCompileError> {
    if !builder.reachable {
        return Err(scalar_compile_error::at(
            syntax.span(),
            ScalarCompileErrorKind::UnreachableStatement,
        ));
    }
    let view = LegacyStatementSyntax::from_node(syntax).map_err(|_| {
        scalar_compile_error::at(syntax.span(), ScalarCompileErrorKind::InvalidStructure)
    })?;
    match view.kind() {
        LegacyStatementKind::Empty => Ok(()),
        LegacyStatementKind::Assignment { target, value } => {
            scalar_local_assignment::assign(builder, view.statement_syntax(), target, value)
        }
        LegacyStatementKind::CompoundAssignment {
            target,
            operator,
            value,
        } => scalar_local_assignment::compound(
            builder,
            view.statement_syntax(),
            target,
            operator,
            value,
        ),
        LegacyStatementKind::Return { value } => {
            if !last {
                return Err(scalar_compile_error::at(
                    view.span(),
                    ScalarCompileErrorKind::ReturnNotLast,
                ));
            }
            queue_return(builder, value, view.statement_syntax().span())?;
            builder.reachable = false;
            Ok(())
        }
        LegacyStatementKind::Update { target, kind } => {
            scalar_local_assignment::postfix(builder, view.statement_syntax(), target, kind)
        }
        LegacyStatementKind::Block { body } => scalar_statement_body::queue(builder, body),
        LegacyStatementKind::If {
            condition,
            consequence,
            alternative,
        } => {
            let span = scalar_body::child(
                view.statement_syntax(),
                0,
                if alternative.is_some() { 7 } else { 5 },
            )?
            .span();
            builder.push_task(
                ScalarTask::BeginIf {
                    consequence,
                    alternative,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(ScalarTask::Visit(condition), condition.span())
        }
        _ => Err(scalar_compile_error::at(
            view.span(),
            ScalarCompileErrorKind::UnsupportedBody,
        )),
    }
}

pub(super) fn queue_return<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    value: Option<LegacySyntaxNode<'tree, 'source>>,
    span: std::ops::Range<u64>,
) -> Result<(), ScalarCompileError> {
    builder.push_task(
        ScalarTask::Emit {
            instruction: KernelInstruction::Return,
            span: span.clone(),
        },
        span.clone(),
    )?;
    if let Some(value) = value {
        builder.push_task(ScalarTask::Visit(value), value.span())
    } else {
        builder.push_task(
            ScalarTask::Emit {
                instruction: KernelInstruction::Constant(KernelValue::Undefined),
                span: span.clone(),
            },
            span,
        )
    }
}
