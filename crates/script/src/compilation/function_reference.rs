// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_field_names;
use crate::compilation::function_local_names;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::expression::{LegacyReferenceKind, LegacyReferenceSyntax};
use crate::syntax::LegacySyntaxNode;

/// Receives the direct reference child of `LegacyPrimaryKind::Reference`.
pub(super) fn read<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let view = LegacyReferenceSyntax::from_node(syntax).map_err(|_| {
        function_compile_error::at(syntax.span(), FunctionCompileErrorKind::InvalidStructure)
    })?;
    match view.kind() {
        LegacyReferenceKind::Name { name } => function_local_names::read(builder, name),
        LegacyReferenceKind::Field { object, name } if builder.mode.bound() => {
            let index = function_field_names::retain(builder, name)?;
            let span = function_body::child(syntax, 1, 3)?.span();
            builder.push_task(
                FunctionTask::Emit {
                    instruction: KernelInstruction::LoadField(index),
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(FunctionTask::Visit(object), object.span())
        }
        LegacyReferenceKind::Index { object, index } => {
            let span = function_body::child(syntax, 1, 4)?.span();
            builder.push_task(
                FunctionTask::Emit {
                    instruction: KernelInstruction::Index,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(FunctionTask::Visit(object), object.span())?;
            builder.push_task(FunctionTask::Visit(index), index.span())
        }
        _ => Err(function_compile_error::at(
            syntax.span(),
            FunctionCompileErrorKind::UnsupportedPrimary,
        )),
    }
}
