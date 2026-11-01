// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_fold_value::FoldValue;
use super::{allocation, invalid, overflow};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::execution::KernelInstruction;
use crate::legacy::{LegacyArithmetic, LegacyComparison, LegacyNumber};
use crate::lexical::LegacyLexeme;
use crate::literal::{LegacyLiteral, LegacyString, LegacyStringKind};

pub(super) fn decode(
    builder: &mut FunctionBuilder<'_, '_>,
    token: LegacyLexeme<'_>,
    kind: LegacyStringKind,
) -> Result<FoldValue, FunctionCompileError> {
    let span = token.span();
    let error =
        |kind| function_compile_error::at(span.clone(), FunctionCompileErrorKind::Literal(kind));
    let work = (token.as_bytes().len() as u64)
        .checked_mul(3)
        .ok_or_else(|| overflow(span.clone()))?;
    builder.ledger.charge(work, span.clone())?;
    let length = LegacyString::decoded_length(
        token.as_bytes(),
        kind,
        builder.ledger.budget.literal.max_string_bytes,
    )
    .map_err(error)?;
    builder.admit_control(length as u64, span.clone())?;
    let LegacyLiteral::String(value) =
        LegacyLiteral::decode(token, builder.ledger.budget.literal).map_err(error)?
    else {
        return Err(invalid(span));
    };
    Ok(FoldValue::String(value))
}

pub(super) fn binary(
    builder: &mut FunctionBuilder<'_, '_>,
    instruction: KernelInstruction,
    left: &LegacyString,
    right: &LegacyString,
    span: Range<u64>,
) -> Result<FoldValue, FunctionCompileError> {
    if left.kind() != right.kind() {
        return Ok(FoldValue::Runtime);
    }
    match instruction {
        KernelInstruction::Compare(
            comparison @ (LegacyComparison::Equal | LegacyComparison::NotEqual),
        ) => {
            builder.ledger.charge(
                if left.as_bytes().len() == right.as_bytes().len() {
                    left.as_bytes().len() as u64
                } else {
                    0
                },
                span,
            )?;
            let equal = left.as_bytes() == right.as_bytes();
            let value = if comparison == LegacyComparison::Equal {
                equal
            } else {
                !equal
            };
            Ok(FoldValue::Number(LegacyNumber::Integer(i32::from(value))))
        }
        KernelInstruction::Arithmetic(LegacyArithmetic::Add)
            if left.kind() == LegacyStringKind::Plain =>
        {
            let length = left
                .as_bytes()
                .len()
                .checked_add(right.as_bytes().len())
                .ok_or_else(|| overflow(span.clone()))?;
            builder.ledger.charge(length as u64, span.clone())?;
            // Inputs remain admitted until the new owned result is complete.
            builder.admit_control(length as u64, span.clone())?;
            left.concatenate(right, length as u64)
                .map(FoldValue::String)
                .map_err(|_| allocation(span))
        }
        _ => Ok(FoldValue::Runtime),
    }
}
