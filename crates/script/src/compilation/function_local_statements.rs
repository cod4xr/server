// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_for;
use crate::compilation::function_local_assignment;
use crate::compilation::function_loop_transfer::{self, FunctionLoopTransfer};
use crate::compilation::function_statement_body;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::function_while;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::{KernelInstruction, KernelValue};
use crate::statement::{LegacyStatementKind, LegacyStatementSyntax};
use crate::syntax::LegacySyntaxNode;

pub(super) fn lower<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
    last: bool,
) -> Result<(), FunctionCompileError> {
    if !builder.reachable {
        return Err(function_compile_error::at(
            syntax.span(),
            FunctionCompileErrorKind::UnreachableStatement,
        ));
    }
    let view = LegacyStatementSyntax::from_node(syntax).map_err(|_| {
        function_compile_error::at(syntax.span(), FunctionCompileErrorKind::InvalidStructure)
    })?;
    match view.kind() {
        LegacyStatementKind::Empty => Ok(()),
        LegacyStatementKind::Assignment { target, value } => {
            function_local_assignment::assign(builder, view.statement_syntax(), target, value)
        }
        LegacyStatementKind::CompoundAssignment {
            target,
            operator,
            value,
        } => function_local_assignment::compound(
            builder,
            view.statement_syntax(),
            target,
            operator,
            value,
        ),
        LegacyStatementKind::Return { value } => {
            if !last {
                return Err(function_compile_error::at(
                    view.span(),
                    FunctionCompileErrorKind::ReturnNotLast,
                ));
            }
            queue_return(builder, value, view.statement_syntax().span())?;
            builder.reachable = false;
            Ok(())
        }
        LegacyStatementKind::Update { target, kind } => {
            function_local_assignment::postfix(builder, view.statement_syntax(), target, kind)
        }
        LegacyStatementKind::Block { body } => function_statement_body::queue(builder, body),
        LegacyStatementKind::If {
            condition,
            consequence,
            alternative,
        } => {
            let span = function_body::child(
                view.statement_syntax(),
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
        LegacyStatementKind::While { condition, body } => {
            let span = function_body::child(view.statement_syntax(), 0, 5)?.span();
            function_while::queue(builder, condition, body, span)
        }
        LegacyStatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => {
            let span = function_body::child(view.statement_syntax(), 0, 8)?.span();
            function_for::queue(builder, initializer, condition, update, body, span)
        }
        LegacyStatementKind::Break => function_loop_transfer::lower(
            builder,
            FunctionLoopTransfer::Break,
            view.statement_syntax().span(),
        ),
        LegacyStatementKind::Continue => function_loop_transfer::lower(
            builder,
            FunctionLoopTransfer::Continue,
            view.statement_syntax().span(),
        ),
        _ => Err(function_compile_error::at(
            view.span(),
            FunctionCompileErrorKind::UnsupportedBody,
        )),
    }
}

pub(super) fn queue_return<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    value: Option<LegacySyntaxNode<'tree, 'source>>,
    span: std::ops::Range<u64>,
) -> Result<(), FunctionCompileError> {
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::Return,
            span: span.clone(),
        },
        span.clone(),
    )?;
    if let Some(value) = value {
        builder.push_task(FunctionTask::Visit(value), value.span())
    } else {
        builder.push_task(
            FunctionTask::Emit {
                instruction: KernelInstruction::Constant(KernelValue::Undefined),
                span: span.clone(),
            },
            span,
        )
    }
}
