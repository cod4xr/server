// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_field_names;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::expression::{LegacyReferenceKind, LegacyReferenceSyntax};
use crate::syntax::LegacySyntaxNode;

pub(super) fn assign<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    value: LegacySyntaxNode<'tree, 'source>,
) -> Result<bool, FunctionCompileError> {
    let Some((object, index)) = field_target(builder, target)? else {
        return Ok(false);
    };
    let span = function_body::child(statement, 1, 3)?.span();
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::StoreField(index),
            span: span.clone(),
        },
        span,
    )?;
    // LIFO tasks evaluate RHS before the receiver, retaining both until publication.
    builder.push_task(FunctionTask::Visit(object), object.span())?;
    builder.push_task(FunctionTask::Visit(value), value.span())?;
    Ok(true)
}

pub(super) fn field_target<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
) -> Result<Option<(LegacySyntaxNode<'tree, 'source>, usize)>, FunctionCompileError> {
    if !builder.mode.bound() {
        return Ok(None);
    }
    let reference = LegacyReferenceSyntax::from_node(target).map_err(|_| {
        function_compile_error::at(target.span(), FunctionCompileErrorKind::InvalidStructure)
    })?;
    let LegacyReferenceKind::Field { object, name } = reference.kind() else {
        return Ok(None);
    };
    let index = function_field_names::retain(builder, name)?;
    Ok(Some((object, index)))
}
