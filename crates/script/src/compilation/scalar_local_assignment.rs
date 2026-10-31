// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::binding::LegacyIdentifier;
use crate::compilation::scalar_body;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_expression_lowering;
use crate::compilation::scalar_local_bindings::ScalarLocalDeclaration;
use crate::compilation::scalar_local_names;
use crate::compilation::scalar_task::ScalarTask;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::expression::{LegacyInfixOperator, LegacyReferenceKind, LegacyReferenceSyntax};
use crate::legacy::LegacyIntegerUpdate;
use crate::lexical::LegacyLexeme;
use crate::statement::LegacyUpdateKind;
use crate::syntax::LegacySyntaxNode;

pub(super) fn assign<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    value: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let token = target_name(target)?;
    let (name, slot) =
        scalar_local_names::declare(builder, token, ScalarLocalDeclaration::Assignment)?;
    let span = scalar_body::child(statement, 1, 3)?.span();
    finish_assignment(builder, name, slot, span)?;
    builder.push_task(ScalarTask::Visit(value), value.span())
}

pub(super) fn compound<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    operator: LegacyInfixOperator,
    value: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let token = target_name(target)?;
    let (name, slot) = scalar_local_names::lookup(builder, token)?;
    builder.emit(KernelInstruction::LoadLocal(slot), token.span())?;
    let span = scalar_body::child(statement, 1, 3)?.span();
    finish_assignment(builder, name, slot, span.clone())?;
    builder.push_task(
        ScalarTask::Emit {
            instruction: scalar_expression_lowering::infix(operator),
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(ScalarTask::Visit(value), value.span())
}

pub(super) fn postfix<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    kind: LegacyUpdateKind,
) -> Result<(), ScalarCompileError> {
    let token = target_name(target)?;
    let (name, slot) = scalar_local_names::lookup(builder, token)?;
    builder.emit(KernelInstruction::LoadLocal(slot), token.span())?;
    let span = scalar_body::child(statement, 1, 2)?.span();
    finish_assignment(builder, name, slot, span.clone())?;
    let update = match kind {
        LegacyUpdateKind::Increment => LegacyIntegerUpdate::Increment,
        LegacyUpdateKind::Decrement => LegacyIntegerUpdate::Decrement,
    };
    builder.push_task(
        ScalarTask::Emit {
            instruction: KernelInstruction::IntegerUpdate(update),
            span: span.clone(),
        },
        span,
    )
}

fn finish_assignment<'source>(
    builder: &mut ScalarBuilder<'_, 'source>,
    name: LegacyIdentifier<'source>,
    slot: usize,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    builder.push_task(
        ScalarTask::FinishAssignment {
            name,
            slot,
            span: span.clone(),
        },
        span,
    )
}

fn target_name<'source>(
    target: LegacySyntaxNode<'_, 'source>,
) -> Result<LegacyLexeme<'source>, ScalarCompileError> {
    let error = || {
        scalar_compile_error::at(
            target.span(),
            ScalarCompileErrorKind::UnsupportedLocalTarget,
        )
    };
    let reference = LegacyReferenceSyntax::from_node(target).map_err(|_| error())?;
    match reference.kind() {
        LegacyReferenceKind::Name { name } => Ok(name),
        _ => Err(error()),
    }
}
