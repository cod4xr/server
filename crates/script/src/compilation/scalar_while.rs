// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_initialization::{self, ScalarInitializationSnapshot};
use crate::compilation::scalar_task::{ScalarLoopHead, ScalarPatchSite, ScalarTask};
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::{KernelBranchCondition, KernelInstruction};
use crate::syntax::LegacySyntaxNode;

pub(super) fn queue<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    condition: LegacySyntaxNode<'tree, 'source>,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let head = ScalarLoopHead(builder.instruction_position());
    builder.push_task(
        ScalarTask::BeginWhileBody {
            head,
            body,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(ScalarTask::Visit(condition), condition.span())
}

pub(super) fn begin_body<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    head: ScalarLoopHead,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let incoming = scalar_initialization::capture(builder, span.clone())?;
    let exit_site = builder.branch(KernelBranchCondition::IfFalse, span.clone())?;
    builder.push_task(
        ScalarTask::AfterWhileBody {
            head,
            exit_site,
            incoming,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(
        ScalarTask::Statement {
            syntax: body,
            last: true,
        },
        body.span(),
    )
}

pub(super) fn after_body(
    builder: &mut ScalarBuilder<'_, '_>,
    head: ScalarLoopHead,
    exit_site: ScalarPatchSite,
    incoming: ScalarInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    if head.0 >= builder.instruction_position() {
        return Err(scalar_compile_error::at(
            span,
            ScalarCompileErrorKind::InvalidBranchBinding,
        ));
    }
    builder.emit(KernelInstruction::Jump(head.0), span.clone())?;
    builder.bind(exit_site, span.clone())?;
    scalar_initialization::restore(builder, incoming, span)
}
