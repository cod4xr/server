// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_initialization;
use crate::compilation::function_loop_transfer::FunctionBreakOwner;
use crate::compilation::function_scopes;
use crate::compilation::function_switch::switch_context::{FunctionSwitchContext, SwitchLabel};
use crate::compilation::function_switch::switch_inventory;
use crate::compilation::function_switch::switch_task::FunctionSwitchTask;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::statement::{LegacyStatementKind, LegacyStatementListSyntax};
use crate::syntax::LegacySyntaxNode;

pub(in crate::compilation) fn queue<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    expression: LegacySyntaxNode<'tree, 'source>,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    builder.push_task(
        FunctionTask::Switch(FunctionSwitchTask::Begin {
            body,
            span: span.clone(),
        }),
        span,
    )?;
    builder.push_task(FunctionTask::Visit(expression), expression.span())
}

pub(super) fn begin<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    builder.emit(KernelInstruction::RequireInteger, span.clone())?;
    let dispatch = builder.jump(span.clone())?;
    let head = dispatch.0;
    let incoming = function_initialization::capture(builder, span.clone())?;
    let body =
        LegacyStatementListSyntax::lower(syntax, builder.ledger.budget.body).map_err(|cause| {
            function_compile_error::at(syntax.span(), FunctionCompileErrorKind::BodyList(cause))
        })?;
    let inventory = switch_inventory::build(builder, &body)?;
    let count = inventory.cases.len();
    let previous_break_owner = builder.break_owner.replace(FunctionBreakOwner::Switch);
    let previous_break_scope = builder.break_scope.take();
    let scope_parent = function_scopes::current(builder);
    let outer = builder.active_switch.replace(FunctionSwitchContext {
        head,
        cases: inventory.cases,
        case_bytes: inventory.bytes,
        default_span: inventory.default_span,
        default_target: None,
        incoming: Some(incoming),
        exit_facts: None,
        breaks: None,
        previous_break_owner,
        previous_break_scope,
        scope_parent,
    });
    builder.reachable = false;
    builder.push_task(
        FunctionTask::Switch(FunctionSwitchTask::End {
            head,
            dispatch,
            outer,
            span: span.clone(),
        }),
        span,
    )?;
    sections(builder, &body, count)
}

fn sections<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    body: &LegacyStatementListSyntax<'tree, 'source>,
    mut count: usize,
) -> Result<(), FunctionCompileError> {
    let mut last = true;
    for syntax in body.statements().rev() {
        builder.ledger.charge(4, syntax.span())?;
        let label = match switch_inventory::classify(syntax)? {
            LegacyStatementKind::Case { .. } => {
                count = count.checked_sub(1).ok_or_else(|| {
                    function_compile_error::at(
                        syntax.span(),
                        FunctionCompileErrorKind::InvalidSwitchContext,
                    )
                })?;
                Some(SwitchLabel::Case(count))
            }
            LegacyStatementKind::Default => Some(SwitchLabel::Default),
            _ => None,
        };
        if let Some(label) = label {
            builder.push_task(
                FunctionTask::Switch(FunctionSwitchTask::Label {
                    label,
                    span: syntax.span(),
                }),
                syntax.span(),
            )?;
            last = true;
        } else {
            builder.push_task(FunctionTask::Statement { syntax, last }, syntax.span())?;
            last = false;
        }
    }
    if count != 0 {
        return Err(function_compile_error::at(
            body.span(),
            FunctionCompileErrorKind::InvalidSwitchContext,
        ));
    }
    Ok(())
}
