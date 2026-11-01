// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::expression::LegacyLiteralPolarity;
use crate::lexical::{LegacyLexeme, LegacyLexemeKind};
use crate::literal::{
    LegacyLiteral, LegacyLiteralError, LegacyLiteralResource, LegacyString, LegacyStringKind,
};

pub(super) fn emit(
    builder: &mut FunctionBuilder<'_, '_>,
    token: LegacyLexeme<'_>,
    polarity: LegacyLiteralPolarity,
    span: Range<u64>,
) -> Result<bool, FunctionCompileError> {
    if !builder.mode.bound()
        || polarity != LegacyLiteralPolarity::Unchanged
        || !matches!(token.kind(), LegacyLexemeKind::Token(code) if code.as_u16() == 0x103)
    {
        return Ok(false);
    }
    let error =
        |kind| function_compile_error::at(token.span(), FunctionCompileErrorKind::Literal(kind));
    let budget = builder.ledger.budget.literal;
    if token.as_bytes().len() as u64 > budget.max_encoded_bytes {
        return Err(error(LegacyLiteralError::BudgetExceeded(
            LegacyLiteralResource::EncodedBytes,
        )));
    }
    let length = LegacyString::decoded_length(
        token.as_bytes(),
        LegacyStringKind::Plain,
        budget.max_string_bytes,
    )
    .map_err(error)?;
    builder.admit_literal(length as u64, span.clone())?;
    builder.ledger.charge(length as u64, span.clone())?;
    if builder.string_literals.len() == builder.string_literals.capacity() {
        builder
            .ledger
            .charge(builder.string_literals.len() as u64, span.clone())?;
    }
    builder.string_literals.try_reserve(1).map_err(|_| {
        function_compile_error::at(span.clone(), FunctionCompileErrorKind::AllocationFailed)
    })?;
    let LegacyLiteral::String(value) = LegacyLiteral::decode(token, budget).map_err(error)? else {
        return Err(function_compile_error::at(
            span,
            FunctionCompileErrorKind::InvalidStructure,
        ));
    };
    if value.kind() != LegacyStringKind::Plain || value.as_bytes().len() != length {
        return Err(function_compile_error::at(
            span,
            FunctionCompileErrorKind::InvalidStructure,
        ));
    }
    let index = builder.string_literals.len();
    builder.string_literals.push(value.into_bytes());
    builder.emit(KernelInstruction::StringLiteral(index), span)?;
    Ok(true)
}
