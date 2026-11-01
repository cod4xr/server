// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::execution::KernelSwitchKey;
use crate::lexical::LegacyLexeme;
use crate::literal::{
    LegacyLiteral, LegacyLiteralError, LegacyLiteralResource, LegacyString, LegacyStringKind,
};
use std::ops::Range;

pub(super) fn retain(
    builder: &mut FunctionBuilder<'_, '_>,
    token: LegacyLexeme<'_>,
) -> Result<KernelSwitchKey, FunctionCompileError> {
    let span = token.span();
    let error =
        |kind| function_compile_error::at(span.clone(), FunctionCompileErrorKind::Literal(kind));
    let budget = builder.ledger.budget.literal;
    let encoded = token.as_bytes().len() as u64;
    if encoded > budget.max_encoded_bytes {
        return Err(error(LegacyLiteralError::BudgetExceeded(
            LegacyLiteralResource::EncodedBytes,
        )));
    }
    // Prepay length preflight plus both decoding passes before inspecting bytes.
    builder.ledger.charge(
        encoded
            .checked_mul(3)
            .ok_or_else(|| overflow(span.clone()))?,
        span.clone(),
    )?;
    let length = LegacyString::decoded_length(
        token.as_bytes(),
        LegacyStringKind::Plain,
        budget.max_string_bytes,
    )
    .map_err(error)?;
    let usage = &mut builder.ledger.usage;
    usage.switch_labels = usage
        .switch_labels
        .checked_add(1)
        .ok_or_else(|| overflow(span.clone()))?;
    usage.retained_switch_bytes = usage
        .retained_switch_bytes
        .checked_add(length as u64)
        .and_then(|bytes| bytes.checked_add(size_of::<Vec<u8>>() as u64))
        .ok_or_else(|| overflow(span.clone()))?;
    builder.admit_tables(span.clone())?;
    if builder.switch_labels.len() == builder.switch_labels.capacity() {
        builder
            .ledger
            .charge(builder.switch_labels.len() as u64, span.clone())?;
    }
    builder.switch_labels.try_reserve(1).map_err(|_| {
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
    let index = builder.switch_labels.len();
    builder.switch_labels.push(value.into_bytes());
    Ok(KernelSwitchKey::StringLabel(index))
}

fn overflow(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::SizeOverflow)
}
