// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_body;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_call;
use crate::compilation::function_compile_error;
use crate::compilation::function_group;
use crate::compilation::function_literal;
use crate::compilation::function_reference;
use crate::compilation::function_reference_value;
use crate::compilation::function_string_literal;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::{KernelBranchCondition, KernelInstruction, KernelScriptRoot};
use crate::expression::{
    LegacyExpressionKind, LegacyExpressionSyntax, LegacyInfixOperator, LegacyLogicalOperator,
    LegacyObjectRoot, LegacyPrefixOperator, LegacyPrimaryKind, LegacyPrimarySyntax,
};
use crate::legacy::{LegacyArithmetic, LegacyComparison, LegacyIntegerOperator};
use crate::syntax::LegacySyntaxNode;

pub(super) fn visit<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let expression = LegacyExpressionSyntax::from_node(node).map_err(|_| {
        function_compile_error::at(node.span(), FunctionCompileErrorKind::InvalidStructure)
    })?;
    match expression.kind() {
        LegacyExpressionKind::Primary { syntax } => primary(builder, syntax),
        LegacyExpressionKind::Prefix { operator, operand } => {
            let span = function_body::child(node, 0, 2)?.span();
            let instruction = match operator {
                LegacyPrefixOperator::LogicalNot => KernelInstruction::LogicalNot,
                LegacyPrefixOperator::BitwiseNot => KernelInstruction::BitwiseNot,
            };
            builder.push_task(
                FunctionTask::Emit {
                    instruction,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(FunctionTask::Visit(operand), operand.span())
        }
        LegacyExpressionKind::Infix {
            operator,
            left,
            right,
        } => {
            let span = function_body::child(node, 1, 3)?.span();
            builder.push_task(
                FunctionTask::Emit {
                    instruction: infix(operator),
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(FunctionTask::Visit(right), right.span())?;
            builder.push_task(FunctionTask::Visit(left), left.span())
        }
        LegacyExpressionKind::Logical {
            operator,
            left,
            right,
        } => {
            let span = function_body::child(node, 1, 3)?.span();
            builder.push_task(
                FunctionTask::BeginLogical {
                    operator,
                    right,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(FunctionTask::Visit(left), left.span())
        }
    }
}

fn primary<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    let primary = LegacyPrimarySyntax::from_node(node).map_err(|_| {
        function_compile_error::at(node.span(), FunctionCompileErrorKind::InvalidStructure)
    })?;
    match primary.kind() {
        LegacyPrimaryKind::Object(root) if builder.mode.bound() => {
            let root = match root {
                LegacyObjectRoot::CurrentObject => KernelScriptRoot::CurrentObject,
                LegacyObjectRoot::Level => KernelScriptRoot::Level,
                LegacyObjectRoot::Game => KernelScriptRoot::Game,
                LegacyObjectRoot::Animation => KernelScriptRoot::Animation,
            };
            builder.emit(KernelInstruction::LoadScriptRoot(root), node.span())
        }
        LegacyPrimaryKind::Reference { syntax } if builder.mode.primitive() => {
            function_reference::read(builder, syntax)
        }
        LegacyPrimaryKind::Call { syntax } if builder.mode.bound() => {
            function_call::queue(builder, syntax)
        }
        LegacyPrimaryKind::FunctionReference { syntax } if builder.mode.bound() => {
            function_reference_value::lower(builder, syntax)
        }
        LegacyPrimaryKind::Size { object } if builder.mode.bound() => {
            let span = function_body::child(node, 1, 2)?.span();
            builder.push_task(
                FunctionTask::Emit {
                    instruction: KernelInstruction::Size,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(FunctionTask::Visit(object), object.span())
        }
        LegacyPrimaryKind::Literal { token, polarity } => {
            builder
                .ledger
                .charge(token.as_bytes().len() as u64, token.span())?;
            if function_string_literal::emit(builder, token, polarity, node.span())? {
                return Ok(());
            }
            let value = function_literal::decode(token, polarity, builder.ledger.budget.literal)?;
            builder.emit(KernelInstruction::Constant(value), node.span())
        }
        LegacyPrimaryKind::Parenthesized { expressions } => {
            function_group::lower(builder, node, expressions)
        }
        _ => Err(function_compile_error::at(
            node.span(),
            FunctionCompileErrorKind::UnsupportedPrimary,
        )),
    }
}

pub(super) fn logical<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    operator: LegacyLogicalOperator,
    right: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    builder.emit(KernelInstruction::CastBool, span.clone())?;
    builder.emit(KernelInstruction::Duplicate, span.clone())?;
    let condition = match operator {
        LegacyLogicalOperator::Or => KernelBranchCondition::IfTrue,
        LegacyLogicalOperator::And => KernelBranchCondition::IfFalse,
    };
    let site = builder.branch(condition, span.clone())?;
    builder.emit(KernelInstruction::Discard, span.clone())?;
    builder.push_task(
        FunctionTask::EndLogical {
            site,
            span: span.clone(),
        },
        span.clone(),
    )?;
    builder.push_task(
        FunctionTask::Emit {
            instruction: KernelInstruction::CastBool,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(FunctionTask::Visit(right), right.span())
}

pub(super) fn infix(operator: LegacyInfixOperator) -> KernelInstruction {
    match operator {
        LegacyInfixOperator::BitwiseOr => KernelInstruction::Integer(LegacyIntegerOperator::Or),
        LegacyInfixOperator::BitwiseExclusiveOr => {
            KernelInstruction::Integer(LegacyIntegerOperator::ExclusiveOr)
        }
        LegacyInfixOperator::BitwiseAnd => KernelInstruction::Integer(LegacyIntegerOperator::And),
        LegacyInfixOperator::Equal => KernelInstruction::Compare(LegacyComparison::Equal),
        LegacyInfixOperator::NotEqual => KernelInstruction::Compare(LegacyComparison::NotEqual),
        LegacyInfixOperator::Less => KernelInstruction::Compare(LegacyComparison::Less),
        LegacyInfixOperator::Greater => KernelInstruction::Compare(LegacyComparison::Greater),
        LegacyInfixOperator::LessOrEqual => {
            KernelInstruction::Compare(LegacyComparison::LessOrEqual)
        }
        LegacyInfixOperator::GreaterOrEqual => {
            KernelInstruction::Compare(LegacyComparison::GreaterOrEqual)
        }
        LegacyInfixOperator::ShiftLeft => {
            KernelInstruction::Integer(LegacyIntegerOperator::ShiftLeft)
        }
        LegacyInfixOperator::ShiftRight => {
            KernelInstruction::Integer(LegacyIntegerOperator::ShiftRight)
        }
        LegacyInfixOperator::Add => KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        LegacyInfixOperator::Subtract => KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
        LegacyInfixOperator::Multiply => KernelInstruction::Arithmetic(LegacyArithmetic::Multiply),
        LegacyInfixOperator::Divide => KernelInstruction::Divide,
        LegacyInfixOperator::Remainder => {
            KernelInstruction::Integer(LegacyIntegerOperator::Remainder)
        }
    }
}
