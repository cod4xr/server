// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_literal;
use crate::compilation::function_switch::switch_context::SwitchCase;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::{KernelSwitchKey, KernelValue};
use crate::expression::{LegacyLiteralPolarity, LegacyPrimaryKind, LegacyPrimarySyntax};
use crate::legacy::LegacyNumber;
use crate::lexical::LegacyLexemeKind;
use crate::statement::{LegacyStatementKind, LegacyStatementListSyntax, LegacyStatementSyntax};
use crate::syntax::LegacySyntaxNode;

pub(super) struct SwitchInventory {
    pub(super) cases: Vec<SwitchCase>,
    pub(super) bytes: u64,
    pub(super) default_span: Option<Range<u64>>,
}

pub(super) fn build(
    builder: &mut FunctionBuilder<'_, '_>,
    body: &LegacyStatementListSyntax<'_, '_>,
) -> Result<SwitchInventory, FunctionCompileError> {
    let error = |kind| function_compile_error::at(body.span(), kind);
    let mut count = 0_usize;
    let mut default_span = None;
    for (index, node) in body.statements().enumerate() {
        builder.ledger.charge(4, node.span())?;
        let kind = classify(node)?;
        match kind {
            LegacyStatementKind::Case { .. } => {
                count = count
                    .checked_add(1)
                    .ok_or_else(|| error(FunctionCompileErrorKind::SizeOverflow))?;
            }
            LegacyStatementKind::Default if default_span.is_some() => {
                return Err(function_compile_error::at(
                    node.span(),
                    FunctionCompileErrorKind::DuplicateCase,
                ));
            }
            LegacyStatementKind::Default => default_span = Some(node.span()),
            _ if index == 0 => return Err(error(FunctionCompileErrorKind::UnsupportedBody)),
            _ => {}
        }
    }
    let bytes = (count as u64)
        .checked_mul(size_of::<SwitchCase>() as u64)
        .ok_or_else(|| error(FunctionCompileErrorKind::SizeOverflow))?;
    builder.admit_control(bytes, body.span())?;
    let mut cases = Vec::new();
    cases
        .try_reserve_exact(count)
        .map_err(|_| error(FunctionCompileErrorKind::AllocationFailed))?;
    for node in body.statements() {
        builder.ledger.charge(4, node.span())?;
        if let LegacyStatementKind::Case { value } = classify(node)? {
            cases.push(SwitchCase {
                key: key(builder, value)?,
                target: None,
                span: node.span(),
            });
        }
    }
    Ok(SwitchInventory {
        cases,
        bytes,
        default_span,
    })
}

pub(super) fn classify<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<LegacyStatementKind<'tree, 'source>, FunctionCompileError> {
    LegacyStatementSyntax::from_node(node)
        .map(LegacyStatementSyntax::kind)
        .map_err(|_| {
            function_compile_error::at(node.span(), FunctionCompileErrorKind::InvalidStructure)
        })
}

fn key(
    builder: &mut FunctionBuilder<'_, '_>,
    node: LegacySyntaxNode<'_, '_>,
) -> Result<KernelSwitchKey, FunctionCompileError> {
    let error = |kind| function_compile_error::at(node.span(), kind);
    let primary = LegacyPrimarySyntax::from_node(node)
        .map_err(|_| error(FunctionCompileErrorKind::InvalidStructure))?;
    let LegacyPrimaryKind::Literal { token, polarity } = primary.kind() else {
        return Err(error(FunctionCompileErrorKind::UnsupportedCase));
    };
    if builder.mode.bound()
        && polarity == LegacyLiteralPolarity::Unchanged
        && matches!(token.kind(), LegacyLexemeKind::Token(code) if code.as_u16() == 0x103)
    {
        return super::switch_labels::retain(builder, token);
    }
    // Literal booleans decode to integers as values, but they are different
    // source productions from integer case labels in the pinned compiler.
    if !matches!(token.kind(), LegacyLexemeKind::Token(code) if code.as_u16() == 0x11f) {
        return Err(error(FunctionCompileErrorKind::UnsupportedCase));
    }
    builder
        .ledger
        .charge(token.as_bytes().len() as u64, token.span())?;
    let value = function_literal::decode(token, polarity, builder.ledger.budget.literal)?;
    match value {
        KernelValue::Number(LegacyNumber::Integer(key)) => Ok(KernelSwitchKey::Integer(key)),
        _ => Err(error(FunctionCompileErrorKind::UnsupportedCase)),
    }
}
