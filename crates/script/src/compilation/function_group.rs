// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::syntax::LegacySyntaxNode;

pub(super) fn lower<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    group: LegacySyntaxNode<'tree, 'source>,
    expressions: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    if function_body::production(expressions) != Some(113) {
        return Err(arity(group));
    }
    let list = function_body::child(expressions, 0, 1)?;
    if function_body::production(list) == Some(112) {
        let inner = function_body::child(list, 0, 1)?;
        return builder.push_task(FunctionTask::Visit(inner), inner.span());
    }
    if !builder.mode.primitive() {
        return Err(arity(group));
    }
    let coordinates = triple(builder, group, list)?;
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::MakeVector,
            span: group.span(),
        },
        group.span(),
    )?;
    // LIFO tasks visit z, y, then x; MakeVector consumes these reversed slots.
    for coordinate in coordinates {
        builder.push_task(FunctionTask::Visit(coordinate), coordinate.span())?;
    }
    Ok(())
}

fn triple<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    group: LegacySyntaxNode<'tree, 'source>,
    list: LegacySyntaxNode<'tree, 'source>,
) -> Result<[LegacySyntaxNode<'tree, 'source>; 3], FunctionCompileError> {
    builder.ledger.charge(1, list.span())?;
    if function_body::production(list) != Some(111) {
        return Err(arity(group));
    }
    let z = function_body::child(list, 2, 3)?;
    let previous = function_body::child(list, 0, 3)?;
    builder.ledger.charge(1, previous.span())?;
    if function_body::production(previous) != Some(111) {
        return Err(arity(group));
    }
    let y = function_body::child(previous, 2, 3)?;
    let first = function_body::child(previous, 0, 3)?;
    builder.ledger.charge(1, first.span())?;
    if function_body::production(first) != Some(112) {
        return Err(arity(group));
    }
    Ok([function_body::child(first, 0, 1)?, y, z])
}

fn arity(group: LegacySyntaxNode<'_, '_>) -> FunctionCompileError {
    function_compile_error::at(
        group.span(),
        FunctionCompileErrorKind::UnsupportedGroupArity,
    )
}
