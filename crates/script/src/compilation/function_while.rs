// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_initialization::{self, FunctionInitializationSnapshot};
use crate::compilation::function_loop_transfer::{
    self, FunctionContinueDestination, FunctionLoopContext,
};
use crate::compilation::function_task::{FunctionLoopHead, FunctionPatchSite, FunctionTask};
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::{KernelBranchCondition, KernelInstruction};
use crate::syntax::LegacySyntaxNode;

pub(super) fn queue<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    condition: LegacySyntaxNode<'tree, 'source>,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let head = FunctionLoopHead(builder.instruction_position());
    builder.push_task(
        FunctionTask::BeginWhileBody {
            head,
            body,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(FunctionTask::Visit(condition), condition.span())
}

pub(super) fn begin_body<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    head: FunctionLoopHead,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let incoming = function_initialization::capture(builder, span.clone())?;
    let exit_site = builder.branch(KernelBranchCondition::IfFalse, span.clone())?;
    let outer_loop =
        function_loop_transfer::enter(builder, head, FunctionContinueDestination::Condition);
    builder.push_task(
        FunctionTask::AfterWhileBody {
            head,
            exit_site,
            incoming,
            outer_loop,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(
        FunctionTask::Statement {
            syntax: body,
            last: true,
        },
        body.span(),
    )
}

pub(super) fn after_body(
    builder: &mut FunctionBuilder<'_, '_>,
    head: FunctionLoopHead,
    exit_site: FunctionPatchSite,
    incoming: FunctionInitializationSnapshot,
    outer_loop: Option<FunctionLoopContext>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    if head.0 >= builder.instruction_position() {
        return Err(function_compile_error::at(
            span,
            FunctionCompileErrorKind::InvalidBranchBinding,
        ));
    }
    builder.emit(KernelInstruction::Jump(head.0), span.clone())?;
    builder.bind(exit_site, span.clone())?;
    function_loop_transfer::leave(builder, head, outer_loop, span.clone())?;
    function_initialization::restore(builder, incoming, span)
}
