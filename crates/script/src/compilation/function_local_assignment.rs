// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::binding::LegacyIdentifier;
use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_expression_lowering;
use crate::compilation::function_field_assignment;
use crate::compilation::function_local_bindings::FunctionLocalDeclaration;
use crate::compilation::function_local_names;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::expression::{LegacyInfixOperator, LegacyReferenceKind, LegacyReferenceSyntax};
use crate::legacy::LegacyIntegerUpdate;
use crate::lexical::LegacyLexeme;
use crate::statement::LegacyUpdateKind;
use crate::syntax::LegacySyntaxNode;

pub(super) fn assign<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    value: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    if function_field_assignment::assign(builder, statement, target, value)? {
        return Ok(());
    }
    let token = target_name(target)?;
    let (name, slot) =
        function_local_names::declare(builder, token, FunctionLocalDeclaration::Assignment)?;
    let span = function_body::child(statement, 1, 3)?.span();
    finish_assignment(builder, name, slot, span)?;
    builder.push_task(FunctionTask::Visit(value), value.span())
}

pub(super) fn compound<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    operator: LegacyInfixOperator,
    value: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let token = target_name(target)?;
    let (name, slot) = function_local_names::lookup(builder, token)?;
    builder.emit(KernelInstruction::LoadLocal(slot), token.span())?;
    let span = function_body::child(statement, 1, 3)?.span();
    finish_assignment(builder, name, slot, span.clone())?;
    builder.push_task(
        FunctionTask::Emit {
            instruction: function_expression_lowering::infix(operator),
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(FunctionTask::Visit(value), value.span())
}

pub(super) fn postfix<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    kind: LegacyUpdateKind,
) -> Result<(), FunctionCompileError> {
    let token = target_name(target)?;
    let (name, slot) = function_local_names::lookup(builder, token)?;
    builder.emit(KernelInstruction::LoadLocal(slot), token.span())?;
    let span = function_body::child(statement, 1, 2)?.span();
    finish_assignment(builder, name, slot, span.clone())?;
    let update = match kind {
        LegacyUpdateKind::Increment => LegacyIntegerUpdate::Increment,
        LegacyUpdateKind::Decrement => LegacyIntegerUpdate::Decrement,
    };
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::IntegerUpdate(update),
            span: span.clone(),
        },
        span,
    )
}

fn finish_assignment<'source>(
    builder: &mut FunctionBuilder<'_, 'source>,
    name: LegacyIdentifier<'source>,
    slot: usize,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    builder.push_task(
        FunctionTask::FinishAssignment {
            name,
            slot,
            span: span.clone(),
        },
        span,
    )
}

fn target_name<'source>(
    target: LegacySyntaxNode<'_, 'source>,
) -> Result<LegacyLexeme<'source>, FunctionCompileError> {
    let error = || {
        function_compile_error::at(
            target.span(),
            FunctionCompileErrorKind::UnsupportedLocalTarget,
        )
    };
    let reference = LegacyReferenceSyntax::from_node(target).map_err(|_| error())?;
    match reference.kind() {
        LegacyReferenceKind::Name { name } => Ok(name),
        _ => Err(error()),
    }
}
