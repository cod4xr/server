// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::scalar_body;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_local_bindings::ScalarLocalDeclaration;
use crate::compilation::scalar_local_names;
use crate::compilation::scalar_task::ScalarTask;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::{KernelInstruction, KernelValue};
use crate::expression::{LegacyReferenceKind, LegacyReferenceSyntax};
use crate::statement::{LegacyStatementKind, LegacyStatementSyntax};
use crate::syntax::LegacySyntaxNode;

pub(super) fn lower<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
    last: bool,
) -> Result<(), ScalarCompileError> {
    let view = LegacyStatementSyntax::from_node(syntax).map_err(|_| {
        scalar_compile_error::at(syntax.span(), ScalarCompileErrorKind::InvalidStructure)
    })?;
    match view.kind() {
        LegacyStatementKind::Empty => Ok(()),
        LegacyStatementKind::Assignment { target, value } => {
            let reference = LegacyReferenceSyntax::from_node(target).map_err(|_| {
                scalar_compile_error::at(
                    target.span(),
                    ScalarCompileErrorKind::UnsupportedLocalTarget,
                )
            })?;
            let LegacyReferenceKind::Name { name: token } = reference.kind() else {
                return Err(scalar_compile_error::at(
                    target.span(),
                    ScalarCompileErrorKind::UnsupportedLocalTarget,
                ));
            };
            let (name, slot) =
                scalar_local_names::declare(builder, token, ScalarLocalDeclaration::Assignment)?;
            let span = scalar_body::child(view.statement_syntax(), 1, 3)?.span();
            builder.push_task(
                ScalarTask::FinishAssignment {
                    name,
                    slot,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(ScalarTask::Visit(value), value.span())
        }
        LegacyStatementKind::Return { value } => {
            if !last {
                return Err(scalar_compile_error::at(
                    view.span(),
                    ScalarCompileErrorKind::ReturnNotLast,
                ));
            }
            queue_return(builder, value, view.statement_syntax().span())
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
