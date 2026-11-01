// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::ScalarCompileError;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_initialization::{self, ScalarInitializationSnapshot};
use crate::compilation::scalar_task::{ScalarPatchSite, ScalarTask};
use crate::execution::KernelBranchCondition;
use crate::syntax::LegacySyntaxNode;

pub(super) fn begin<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    consequence: LegacySyntaxNode<'tree, 'source>,
    alternative: Option<LegacySyntaxNode<'tree, 'source>>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let incoming = scalar_initialization::capture(builder, span.clone())?;
    let false_site = builder.branch(KernelBranchCondition::IfFalse, span.clone())?;
    builder.push_task(
        ScalarTask::AfterThen {
            false_site,
            incoming,
            alternative,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(
        ScalarTask::Statement {
            syntax: consequence,
            last: true,
        },
        consequence.span(),
    )
}

pub(super) fn after_then<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    false_site: ScalarPatchSite,
    incoming: ScalarInitializationSnapshot,
    alternative: Option<LegacySyntaxNode<'tree, 'source>>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    let Some(alternative) = alternative else {
        builder.bind(false_site, span.clone())?;
        return scalar_initialization::merge(builder, incoming, span);
    };
    let then_facts = scalar_initialization::capture(builder, span.clone())?;
    let end_site = builder.jump(span.clone())?;
    builder.bind(false_site, span.clone())?;
    scalar_initialization::restore(builder, incoming, span.clone())?;
    builder.push_task(
        ScalarTask::AfterElse {
            end_site,
            then_facts,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(
        ScalarTask::Statement {
            syntax: alternative,
            last: true,
        },
        alternative.span(),
    )
}

pub(super) fn after_else(
    builder: &mut ScalarBuilder<'_, '_>,
    end_site: ScalarPatchSite,
    then_facts: ScalarInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    scalar_initialization::merge(builder, then_facts, span.clone())?;
    builder.bind(end_site, span)
}
