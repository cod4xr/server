// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_for_clause;
use crate::compilation::function_for_update;
use crate::compilation::function_initialization::{self, FunctionInitializationSnapshot};
use crate::compilation::function_loop_transfer::{
    self, FunctionContinueDestination, FunctionLoopContext,
};
use crate::compilation::function_task::{FunctionLoopHead, FunctionPatchSite, FunctionTask};
use crate::compilation::function_while;
use crate::execution::{KernelBranchCondition, KernelInstruction, KernelValue};
use crate::legacy::LegacyNumber;
use crate::syntax::LegacySyntaxNode;

pub(super) struct FunctionForPlan<'tree, 'source> {
    condition: Option<LegacySyntaxNode<'tree, 'source>>,
    update: Option<LegacySyntaxNode<'tree, 'source>>,
    body: LegacySyntaxNode<'tree, 'source>,
    pub(super) span: Range<u64>,
}

pub(super) struct FunctionForEnd {
    head: FunctionLoopHead,
    exit_site: FunctionPatchSite,
    incoming: FunctionInitializationSnapshot,
    outer_loop: Option<FunctionLoopContext>,
    pub(super) span: Range<u64>,
}

pub(super) fn queue<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    initializer: LegacySyntaxNode<'tree, 'source>,
    condition: LegacySyntaxNode<'tree, 'source>,
    update: LegacySyntaxNode<'tree, 'source>,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    function_for_clause::validate(initializer)?;
    let plan = FunctionForPlan {
        condition: function_for_clause::condition(condition)?,
        update: function_for_clause::update(update)?,
        body,
        span: span.clone(),
    };
    builder.push_task(FunctionTask::AfterForInit(plan), span)?;
    builder.push_task(
        FunctionTask::Statement {
            syntax: initializer,
            last: true,
        },
        initializer.span(),
    )
}

pub(super) fn after_init<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    plan: FunctionForPlan<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let head = FunctionLoopHead(builder.instruction_position());
    let condition = plan.condition;
    let span = plan.span.clone();
    builder.push_task(FunctionTask::BeginForBody { head, plan }, span.clone())?;
    if let Some(condition) = condition {
        builder.push_task(FunctionTask::Visit(condition), condition.span())
    } else {
        builder.push_task(
            FunctionTask::Emit {
                instruction: KernelInstruction::Constant(KernelValue::Number(
                    LegacyNumber::Integer(1),
                )),
                span: span.clone(),
            },
            span,
        )
    }
}

pub(super) fn begin_body<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    head: FunctionLoopHead,
    plan: FunctionForPlan<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let incoming = function_initialization::capture(builder, plan.span.clone())?;
    let exit_site = builder.branch(KernelBranchCondition::IfFalse, plan.span.clone())?;
    let outer_loop = function_loop_transfer::enter(
        builder,
        head,
        FunctionContinueDestination::Update {
            sites: None,
            facts: None,
        },
    );
    let end = FunctionForEnd {
        head,
        exit_site,
        incoming,
        outer_loop,
        span: plan.span.clone(),
    };
    builder.push_task(
        FunctionTask::AfterForBody {
            end,
            update: plan.update,
        },
        plan.span,
    )?;
    builder.push_task(
        FunctionTask::Statement {
            syntax: plan.body,
            last: true,
        },
        plan.body.span(),
    )
}

pub(super) fn after_body<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    end: FunctionForEnd,
    update: Option<LegacySyntaxNode<'tree, 'source>>,
) -> Result<(), FunctionCompileError> {
    function_for_update::prepare(builder, &end.incoming, end.span.clone())?;
    let span = end.span.clone();
    builder.push_task(FunctionTask::AfterForUpdate(end), span)?;
    if let Some(update) = update {
        builder.push_task(
            FunctionTask::Statement {
                syntax: update,
                last: true,
            },
            update.span(),
        )?;
    }
    Ok(())
}

pub(super) fn after_update(
    builder: &mut FunctionBuilder<'_, '_>,
    end: FunctionForEnd,
) -> Result<(), FunctionCompileError> {
    function_while::after_body(
        builder,
        end.head,
        end.exit_site,
        end.incoming,
        end.outer_loop,
        end.span,
    )
}
