// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_initialization::{self, FunctionInitializationSnapshot};
use crate::compilation::function_scopes;
use crate::compilation::function_task::{FunctionPatchSite, FunctionTask};
use crate::execution::KernelBranchCondition;
use crate::syntax::LegacySyntaxNode;

pub(super) fn queue<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
    condition: LegacySyntaxNode<'tree, 'source>,
    consequence: LegacySyntaxNode<'tree, 'source>,
    alternative: Option<LegacySyntaxNode<'tree, 'source>>,
) -> Result<(), FunctionCompileError> {
    let span = crate::compilation::function_body::child(
        syntax,
        0,
        if alternative.is_some() { 7 } else { 5 },
    )?
    .span();
    builder.push_task(
        FunctionTask::BeginIf {
            consequence,
            alternative,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(FunctionTask::Visit(condition), condition.span())
}

pub(super) fn begin<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    consequence: LegacySyntaxNode<'tree, 'source>,
    alternative: Option<LegacySyntaxNode<'tree, 'source>>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let incoming = function_initialization::capture(builder, span.clone())?;
    let false_site = builder.branch(KernelBranchCondition::IfFalse, span.clone())?;
    function_scopes::begin_if(builder, span.clone())?;
    builder.push_task(
        FunctionTask::AfterThen {
            false_site,
            incoming,
            alternative,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(
        FunctionTask::Statement {
            syntax: consequence,
            last: true,
        },
        consequence.span(),
    )
}

pub(super) fn after_then<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    false_site: FunctionPatchSite,
    incoming: FunctionInitializationSnapshot,
    alternative: Option<LegacySyntaxNode<'tree, 'source>>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    function_scopes::end_consequence(builder, span.clone())?;
    let Some(alternative) = alternative else {
        builder.bind(false_site, span.clone())?;
        return function_initialization::merge(builder, incoming, span);
    };
    let then_facts = function_initialization::capture(builder, span.clone())?;
    let end_site = builder.jump(span.clone())?;
    builder.bind(false_site, span.clone())?;
    function_initialization::restore(builder, incoming, span.clone())?;
    function_scopes::begin_else(builder, span.clone())?;
    builder.push_task(
        FunctionTask::AfterElse {
            end_site,
            then_facts,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(
        FunctionTask::Statement {
            syntax: alternative,
            last: true,
        },
        alternative.span(),
    )
}

pub(super) fn after_else(
    builder: &mut FunctionBuilder<'_, '_>,
    end_site: FunctionPatchSite,
    then_facts: FunctionInitializationSnapshot,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    function_scopes::end_if(builder, then_facts.reachable(), span.clone())?;
    function_initialization::merge(builder, then_facts, span.clone())?;
    builder.bind(end_site, span)
}
