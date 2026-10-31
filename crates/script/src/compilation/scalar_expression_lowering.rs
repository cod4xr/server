// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_body;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_literal;
use crate::compilation::scalar_local_names;
use crate::compilation::scalar_task::ScalarTask;
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::{KernelBranchCondition, KernelInstruction};
use crate::expression::{
    LegacyExpressionKind, LegacyExpressionSyntax, LegacyInfixOperator, LegacyLogicalOperator,
    LegacyPrefixOperator, LegacyPrimaryKind, LegacyPrimarySyntax,
};
use crate::legacy::{LegacyArithmetic, LegacyComparison, LegacyIntegerOperator};
use crate::syntax::LegacySyntaxNode;

pub(super) fn visit<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let expression = LegacyExpressionSyntax::from_node(node).map_err(|_| {
        scalar_compile_error::at(node.span(), ScalarCompileErrorKind::InvalidStructure)
    })?;
    match expression.kind() {
        LegacyExpressionKind::Primary { syntax } => primary(builder, syntax),
        LegacyExpressionKind::Prefix { operator, operand } => {
            let span = scalar_body::child(node, 0, 2)?.span();
            let instruction = match operator {
                LegacyPrefixOperator::LogicalNot => KernelInstruction::LogicalNot,
                LegacyPrefixOperator::BitwiseNot => KernelInstruction::BitwiseNot,
            };
            builder.push_task(
                ScalarTask::Emit {
                    instruction,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(ScalarTask::Visit(operand), operand.span())
        }
        LegacyExpressionKind::Infix {
            operator,
            left,
            right,
        } => {
            let span = scalar_body::child(node, 1, 3)?.span();
            builder.push_task(
                ScalarTask::Emit {
                    instruction: infix(operator),
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(ScalarTask::Visit(right), right.span())?;
            builder.push_task(ScalarTask::Visit(left), left.span())
        }
        LegacyExpressionKind::Logical {
            operator,
            left,
            right,
        } => {
            let span = scalar_body::child(node, 1, 3)?.span();
            builder.push_task(
                ScalarTask::BeginLogical {
                    operator,
                    right,
                    span: span.clone(),
                },
                span,
            )?;
            builder.push_task(ScalarTask::Visit(left), left.span())
        }
    }
}

fn primary<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let primary = LegacyPrimarySyntax::from_node(node).map_err(|_| {
        scalar_compile_error::at(node.span(), ScalarCompileErrorKind::InvalidStructure)
    })?;
    match primary.kind() {
        LegacyPrimaryKind::Reference { syntax } if builder.locals.is_some() => {
            scalar_local_names::read(builder, syntax)
        }
        LegacyPrimaryKind::Literal { token, polarity } => {
            builder
                .ledger
                .charge(token.as_bytes().len() as u64, token.span())?;
            let value = scalar_literal::decode(token, polarity, builder.ledger.budget.literal)?;
            builder.emit(KernelInstruction::Constant(value), node.span())
        }
        LegacyPrimaryKind::Parenthesized { expressions } => {
            if scalar_body::production(expressions) != Some(113) {
                return Err(scalar_compile_error::at(
                    node.span(),
                    ScalarCompileErrorKind::UnsupportedGroupArity,
                ));
            }
            let list = scalar_body::child(expressions, 0, 1)?;
            if scalar_body::production(list) != Some(112) {
                return Err(scalar_compile_error::at(
                    node.span(),
                    ScalarCompileErrorKind::UnsupportedGroupArity,
                ));
            }
            let inner = scalar_body::child(list, 0, 1)?;
            builder.push_task(ScalarTask::Visit(inner), inner.span())
        }
        _ => Err(scalar_compile_error::at(
            node.span(),
            ScalarCompileErrorKind::UnsupportedPrimary,
        )),
    }
}

pub(super) fn logical<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    operator: LegacyLogicalOperator,
    right: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    builder.emit(KernelInstruction::CastBool, span.clone())?;
    builder.emit(KernelInstruction::Duplicate, span.clone())?;
    let condition = match operator {
        LegacyLogicalOperator::Or => KernelBranchCondition::IfTrue,
        LegacyLogicalOperator::And => KernelBranchCondition::IfFalse,
    };
    let site = builder.branch(condition, span.clone())?;
    builder.emit(KernelInstruction::Discard, span.clone())?;
    builder.push_task(
        ScalarTask::EndLogical {
            site,
            span: span.clone(),
        },
        span.clone(),
    )?;
    builder.push_task(
        ScalarTask::Emit {
            instruction: KernelInstruction::CastBool,
            span: span.clone(),
        },
        span,
    )?;
    builder.push_task(ScalarTask::Visit(right), right.span())
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
