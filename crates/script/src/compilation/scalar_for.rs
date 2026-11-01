// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::ScalarCompileError;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_for_clause;
use crate::compilation::scalar_for_update;
use crate::compilation::scalar_initialization::{self, ScalarInitializationSnapshot};
use crate::compilation::scalar_loop_transfer::{
    self, ScalarContinueDestination, ScalarLoopContext,
};
use crate::compilation::scalar_task::{ScalarLoopHead, ScalarPatchSite, ScalarTask};
use crate::compilation::scalar_while;
use crate::execution::{KernelBranchCondition, KernelInstruction, KernelValue};
use crate::legacy::LegacyNumber;
use crate::syntax::LegacySyntaxNode;

pub(super) struct ScalarForPlan<'tree, 'source> {
    condition: Option<LegacySyntaxNode<'tree, 'source>>,
    update: Option<LegacySyntaxNode<'tree, 'source>>,
    body: LegacySyntaxNode<'tree, 'source>,
    pub(super) span: Range<u64>,
}

pub(super) struct ScalarForEnd {
    head: ScalarLoopHead,
    exit_site: ScalarPatchSite,
    incoming: ScalarInitializationSnapshot,
    outer_loop: Option<ScalarLoopContext>,
    pub(super) span: Range<u64>,
}

pub(super) fn queue<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    initializer: LegacySyntaxNode<'tree, 'source>,
    condition: LegacySyntaxNode<'tree, 'source>,
    update: LegacySyntaxNode<'tree, 'source>,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    scalar_for_clause::validate(initializer)?;
    let plan = ScalarForPlan {
        condition: scalar_for_clause::condition(condition)?,
        update: scalar_for_clause::update(update)?,
        body,
        span: span.clone(),
    };
    builder.push_task(ScalarTask::AfterForInit(plan), span)?;
    builder.push_task(
        ScalarTask::Statement {
            syntax: initializer,
            last: true,
        },
        initializer.span(),
    )
}

pub(super) fn after_init<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    plan: ScalarForPlan<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let head = ScalarLoopHead(builder.instruction_position());
    let condition = plan.condition;
    let span = plan.span.clone();
    builder.push_task(ScalarTask::BeginForBody { head, plan }, span.clone())?;
    if let Some(condition) = condition {
        builder.push_task(ScalarTask::Visit(condition), condition.span())
    } else {
        builder.push_task(
            ScalarTask::Emit {
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
    builder: &mut ScalarBuilder<'tree, 'source>,
    head: ScalarLoopHead,
    plan: ScalarForPlan<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let incoming = scalar_initialization::capture(builder, plan.span.clone())?;
    let exit_site = builder.branch(KernelBranchCondition::IfFalse, plan.span.clone())?;
    let outer_loop = scalar_loop_transfer::enter(
        builder,
        head,
        ScalarContinueDestination::Update {
            sites: None,
            facts: None,
        },
    );
    let end = ScalarForEnd {
        head,
        exit_site,
        incoming,
        outer_loop,
        span: plan.span.clone(),
    };
    builder.push_task(
        ScalarTask::AfterForBody {
            end,
            update: plan.update,
        },
        plan.span,
    )?;
    builder.push_task(
        ScalarTask::Statement {
            syntax: plan.body,
            last: true,
        },
        plan.body.span(),
    )
}

pub(super) fn after_body<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    end: ScalarForEnd,
    update: Option<LegacySyntaxNode<'tree, 'source>>,
) -> Result<(), ScalarCompileError> {
    scalar_for_update::prepare(builder, &end.incoming, end.span.clone())?;
    let span = end.span.clone();
    builder.push_task(ScalarTask::AfterForUpdate(end), span)?;
    if let Some(update) = update {
        builder.push_task(
            ScalarTask::Statement {
                syntax: update,
                last: true,
            },
            update.span(),
        )?;
    }
    Ok(())
}

pub(super) fn after_update(
    builder: &mut ScalarBuilder<'_, '_>,
    end: ScalarForEnd,
) -> Result<(), ScalarCompileError> {
    scalar_while::after_body(
        builder,
        end.head,
        end.exit_site,
        end.incoming,
        end.outer_loop,
        end.span,
    )
}
