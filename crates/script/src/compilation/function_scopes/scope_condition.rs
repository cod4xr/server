// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_fold_string;
use super::scope_fold_value::{self, FoldValue};
use super::{allocation, invalid, overflow};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::compilation::{function_body, function_group, function_literal};
use crate::execution::KernelValue;
use crate::expression::{
    LegacyExpressionKind, LegacyExpressionSyntax, LegacyInfixOperator, LegacyPrimaryKind,
    LegacyPrimarySyntax,
};
use crate::lexical::LegacyLexemeKind;
use crate::literal::LegacyStringKind;
use crate::syntax::LegacySyntaxNode;

#[derive(Clone, Copy)]
pub(super) enum ScopeCondition {
    Dynamic,
    Always(bool),
}

enum FoldTask<'tree, 'source> {
    Evaluate(LegacySyntaxNode<'tree, 'source>),
    Binary(LegacyInfixOperator),
    Vector,
}

pub(super) fn classify(
    builder: &mut FunctionBuilder<'_, '_>,
    syntax: LegacySyntaxNode<'_, '_>,
    outer_tasks: usize,
) -> Result<ScopeCondition, FunctionCompileError> {
    let span = syntax.span();
    let mut tasks = Vec::new();
    let mut values = Vec::new();
    push(
        builder,
        &mut tasks,
        FoldTask::Evaluate(syntax),
        outer_tasks,
        span.clone(),
    )?;
    while let Some(task) = tasks.pop() {
        builder.ledger.charge(1, span.clone())?;
        let value = match task {
            FoldTask::Evaluate(node) => evaluate(
                builder,
                node,
                &mut tasks,
                outer_tasks
                    .checked_add(1)
                    .ok_or_else(|| overflow(span.clone()))?,
                span.clone(),
            )?,
            FoldTask::Vector => {
                let x = pop(builder, &mut values, span.clone())?;
                let y = pop(builder, &mut values, span.clone())?;
                let z = pop(builder, &mut values, span.clone())?;
                Some(scope_fold_value::vector(builder, [x, y, z], span.clone())?)
            }
            FoldTask::Binary(operator) => {
                let right = pop(builder, &mut values, span.clone())?;
                let left = pop(builder, &mut values, span.clone())?;
                Some(scope_fold_value::binary(
                    builder,
                    operator,
                    left,
                    right,
                    span.clone(),
                )?)
            }
        };
        if let Some(value) = value {
            builder.admit_control(size_of::<FoldValue>() as u64, span.clone())?;
            values
                .try_reserve(1)
                .map_err(|_| allocation(span.clone()))?;
            values.push(value);
        }
        builder
            .ledger
            .release_control(size_of::<FoldTask<'_, '_>>() as u64, span.clone())?;
    }
    if values.len() != 1 {
        return Err(invalid(span));
    }
    let value = pop(builder, &mut values, span.clone())?;
    let result = match &value {
        FoldValue::Number(value) => ScopeCondition::Always(value.as_bool()),
        _ => ScopeCondition::Dynamic,
    };
    scope_fold_value::release(builder, value, span)?;
    Ok(result)
}

fn evaluate<'tree, 'source>(
    builder: &mut FunctionBuilder<'_, '_>,
    node: LegacySyntaxNode<'tree, 'source>,
    tasks: &mut Vec<FoldTask<'tree, 'source>>,
    outer_tasks: usize,
    span: Range<u64>,
) -> Result<Option<FoldValue>, FunctionCompileError> {
    let expression = LegacyExpressionSyntax::from_node(node).map_err(|_| invalid(node.span()))?;
    let value = match expression.kind() {
        LegacyExpressionKind::Primary { syntax } => {
            return primary(builder, syntax, tasks, outer_tasks, span);
        }
        LegacyExpressionKind::Infix {
            operator,
            left,
            right,
        } => {
            push(
                builder,
                tasks,
                FoldTask::Binary(operator),
                outer_tasks,
                span.clone(),
            )?;
            push(
                builder,
                tasks,
                FoldTask::Evaluate(right),
                outer_tasks,
                span.clone(),
            )?;
            push(builder, tasks, FoldTask::Evaluate(left), outer_tasks, span)?;
            return Ok(None);
        }
        LegacyExpressionKind::Prefix { .. } | LegacyExpressionKind::Logical { .. } => {
            FoldValue::Runtime
        }
    };
    Ok(Some(value))
}

fn primary<'tree, 'source>(
    builder: &mut FunctionBuilder<'_, '_>,
    node: LegacySyntaxNode<'tree, 'source>,
    tasks: &mut Vec<FoldTask<'tree, 'source>>,
    outer_tasks: usize,
    span: Range<u64>,
) -> Result<Option<FoldValue>, FunctionCompileError> {
    let primary = LegacyPrimarySyntax::from_node(node).map_err(|_| invalid(node.span()))?;
    let value = match primary.kind() {
        LegacyPrimaryKind::Literal { token, polarity } => {
            builder
                .ledger
                .charge(token.as_bytes().len() as u64, token.span())?;
            if matches!(token.kind(), LegacyLexemeKind::Token(code) if matches!(code.as_u16(), 0x103 | 0x104))
            {
                let kind = if matches!(token.kind(), LegacyLexemeKind::Token(code) if code.as_u16() == 0x103)
                {
                    LegacyStringKind::Plain
                } else {
                    LegacyStringKind::Localized
                };
                scope_fold_string::decode(builder, token, kind)?
            } else {
                match function_literal::decode(token, polarity, builder.ledger.budget.literal)? {
                    KernelValue::Number(value) => FoldValue::Number(value),
                    KernelValue::Undefined => FoldValue::Undefined,
                    _ => FoldValue::Runtime,
                }
            }
        }
        LegacyPrimaryKind::Parenthesized { expressions } => {
            if function_body::production(expressions) != Some(113) {
                return Err(function_compile_error::at(
                    node.span(),
                    FunctionCompileErrorKind::UnsupportedGroupArity,
                ));
            }
            let list = function_body::child(expressions, 0, 1)?;
            if function_body::production(list) == Some(112) {
                let inner = function_body::child(list, 0, 1)?;
                push(builder, tasks, FoldTask::Evaluate(inner), outer_tasks, span)?;
            } else {
                let coordinates = function_group::triple(builder, node, list)?;
                push(builder, tasks, FoldTask::Vector, outer_tasks, span.clone())?;
                for coordinate in coordinates {
                    push(
                        builder,
                        tasks,
                        FoldTask::Evaluate(coordinate),
                        outer_tasks,
                        span.clone(),
                    )?;
                }
            }
            return Ok(None);
        }
        _ => FoldValue::Runtime,
    };
    Ok(Some(value))
}

fn push<'tree, 'source>(
    builder: &mut FunctionBuilder<'_, '_>,
    tasks: &mut Vec<FoldTask<'tree, 'source>>,
    task: FoldTask<'tree, 'source>,
    outer_tasks: usize,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let count = tasks
        .len()
        .checked_add(1)
        .and_then(|count| count.checked_add(outer_tasks))
        .ok_or_else(|| overflow(span.clone()))?;
    builder.admit_control(size_of::<FoldTask<'_, '_>>() as u64, span.clone())?;
    builder.ledger.auxiliary_frames(count, span.clone())?;
    tasks.try_reserve(1).map_err(|_| allocation(span))?;
    tasks.push(task);
    Ok(())
}

fn pop(
    builder: &mut FunctionBuilder<'_, '_>,
    values: &mut Vec<FoldValue>,
    span: Range<u64>,
) -> Result<FoldValue, FunctionCompileError> {
    let value = values.pop().ok_or_else(|| invalid(span.clone()))?;
    builder
        .ledger
        .release_control(size_of::<FoldValue>() as u64, span)?;
    Ok(value)
}
