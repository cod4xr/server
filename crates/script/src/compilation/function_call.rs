// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_call_arguments;
use crate::compilation::function_call_target;
use crate::compilation::function_compile_error;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::expression::{
    ExpressionListCursor, LegacyCallSyntax, LegacyCalleeKind, LegacyInvocationKind,
};
use crate::syntax::LegacySyntaxNode;

pub(super) fn statement<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::Discard,
            span: node.span(),
        },
        node.span(),
    )?;
    queue(builder, node)
}

pub(super) fn queue<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let error = |kind| function_compile_error::at(node.span(), kind);
    let call = LegacyCallSyntax::from_node(node)
        .map_err(|_| error(FunctionCompileErrorKind::InvalidStructure))?;
    if call.receiver().is_some() || call.kind() != LegacyInvocationKind::Immediate {
        return Err(error(FunctionCompileErrorKind::UnsupportedCall));
    }
    let discarded = crate::compilation::function_developer::discarded(builder).is_some();
    let target = match call.callee() {
        LegacyCalleeKind::Named(callee) if discarded => {
            crate::compilation::function_developer::validate_name(builder, callee)?;
            None
        }
        LegacyCalleeKind::Named(callee) => Some(function_call_target::named_call(builder, callee)?),
        LegacyCalleeKind::Indirect { .. } => None,
    };
    let arguments = call.arguments_syntax();
    let count = function_call_arguments::count(builder, arguments)?;
    let finish = if discarded {
        FunctionTask::ValidationOnly { span: node.span() }
    } else {
        FunctionTask::Emit {
            instruction: match target {
                Some(target) => target.instruction(count),
                None => KernelInstruction::CallIndirect { arguments: count },
            },
            span: node.span(),
        }
    };
    builder.push_task(finish, node.span())?;
    if let LegacyCalleeKind::Indirect { expression } = call.callee() {
        builder.push_task(FunctionTask::Visit(expression), expression.span())?;
    }
    builder.push_task(
        FunctionTask::CallArguments {
            cursor: ExpressionListCursor::new(arguments),
            span: arguments.span(),
        },
        arguments.span(),
    )
}
