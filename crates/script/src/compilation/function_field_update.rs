// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::FunctionCompileError;
use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_expression_lowering;
use crate::compilation::function_field_assignment;
use crate::compilation::function_task::FunctionTask;
use crate::execution::KernelInstruction;
use crate::expression::LegacyInfixOperator;
use crate::legacy::LegacyIntegerUpdate;
use crate::statement::LegacyUpdateKind;
use crate::syntax::LegacySyntaxNode;

pub(super) fn compound<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    operator: LegacyInfixOperator,
    value: LegacySyntaxNode<'tree, 'source>,
) -> Result<bool, FunctionCompileError> {
    let Some((object, index)) = function_field_assignment::field_target(builder, target)? else {
        return Ok(false);
    };
    let span = function_body::child(statement, 1, 3)?.span();
    // Native compound lowering reads the target, evaluates RHS/operator, then
    // evaluates the writable target again. Receiver side effects occur twice.
    for task in [
        FunctionTask::Emit {
            instruction: KernelInstruction::StoreField(index),
            span: span.clone(),
        },
        FunctionTask::Visit(object),
        FunctionTask::Emit {
            instruction: function_expression_lowering::infix(operator),
            span: span.clone(),
        },
        FunctionTask::Visit(value),
        FunctionTask::Emit {
            instruction: KernelInstruction::LoadField(index),
            span: span.clone(),
        },
        FunctionTask::Visit(object),
    ] {
        builder.push_task(task, span.clone())?;
    }
    Ok(true)
}

pub(super) fn postfix<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    kind: LegacyUpdateKind,
) -> Result<bool, FunctionCompileError> {
    let Some((object, index)) = function_field_assignment::field_target(builder, target)? else {
        return Ok(false);
    };
    let span = function_body::child(statement, 1, 2)?.span();
    let update = match kind {
        LegacyUpdateKind::Increment => LegacyIntegerUpdate::Increment,
        LegacyUpdateKind::Decrement => LegacyIntegerUpdate::Decrement,
    };
    // LIFO: receiver, duplicate, read, integer update, swap, store. Keep the
    // selected receiver rooted throughout; do not reevaluate its expression.
    for instruction in [
        KernelInstruction::StoreField(index),
        KernelInstruction::Swap,
        KernelInstruction::IntegerUpdate(update),
        KernelInstruction::LoadField(index),
        KernelInstruction::Duplicate,
    ] {
        builder.push_task(
            FunctionTask::Emit {
                instruction,
                span: span.clone(),
            },
            span.clone(),
        )?;
    }
    builder.push_task(FunctionTask::Visit(object), object.span())?;
    Ok(true)
}
