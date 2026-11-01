// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{LegacyFunctionVisibility, LegacyIdentifier};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_call_arguments;
use crate::compilation::function_compile_error;
use crate::compilation::function_lowering_mode::FunctionLoweringMode;
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
    let LegacyCalleeKind::Named(callee) = call.callee() else {
        return Err(error(FunctionCompileErrorKind::UnsupportedCall));
    };
    if callee.module().is_some() {
        return Err(error(FunctionCompileErrorKind::UnsupportedCall));
    }
    let token = callee.name();
    builder
        .ledger
        .identifier(token.as_bytes().len() as u64, token.span())?;
    let name = LegacyIdentifier::from_lexeme(token).map_err(|cause| {
        function_compile_error::at(token.span(), FunctionCompileErrorKind::Identifier(cause))
    })?;
    let FunctionLoweringMode::BoundModule { scope, caller } = builder.mode else {
        return Err(error(FunctionCompileErrorKind::InvalidStructure));
    };
    let target = scope.find(name).ok_or_else(|| {
        function_compile_error::at(token.span(), FunctionCompileErrorKind::UnknownFunction)
    })?;
    if caller == LegacyFunctionVisibility::Ordinary
        && target.visibility == LegacyFunctionVisibility::DeveloperOnly
    {
        return Err(function_compile_error::at(
            token.span(),
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ));
    }
    let arguments = call.arguments_syntax();
    let count = function_call_arguments::count(builder, arguments)?;
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::Call {
                function: target.function,
                arguments: count,
            },
            span: node.span(),
        },
        node.span(),
    )?;
    builder.push_task(
        FunctionTask::CallArguments {
            cursor: ExpressionListCursor::new(arguments),
            span: arguments.span(),
        },
        arguments.span(),
    )
}
