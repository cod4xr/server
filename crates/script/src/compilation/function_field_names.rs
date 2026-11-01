// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::LegacyIdentifier;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::lexical::LegacyLexeme;

pub(super) fn retain(
    builder: &mut FunctionBuilder<'_, '_>,
    token: LegacyLexeme<'_>,
) -> Result<usize, FunctionCompileError> {
    let error = |kind| function_compile_error::at(token.span(), kind);
    let length = token.as_bytes().len() as u64;
    builder.ledger.identifier(length, token.span())?;
    let name = LegacyIdentifier::from_lexeme(token)
        .map_err(|cause| error(FunctionCompileErrorKind::Identifier(cause)))?;
    builder.ledger.usage.field_names = builder
        .ledger
        .usage
        .field_names
        .checked_add(1)
        .ok_or_else(|| error(FunctionCompileErrorKind::SizeOverflow))?;
    builder.ledger.usage.retained_field_bytes = builder
        .ledger
        .usage
        .retained_field_bytes
        .checked_add(length)
        .and_then(|bytes| bytes.checked_add(size_of::<Vec<u8>>() as u64))
        .ok_or_else(|| error(FunctionCompileErrorKind::SizeOverflow))?;
    builder.admit_tables(token.span())?;
    builder.ledger.charge(length, token.span())?;
    if builder.field_names.len() == builder.field_names.capacity() {
        builder
            .ledger
            .charge(builder.field_names.len() as u64, token.span())?;
    }
    builder
        .field_names
        .try_reserve(1)
        .map_err(|_| error(FunctionCompileErrorKind::AllocationFailed))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(token.as_bytes().len())
        .map_err(|_| error(FunctionCompileErrorKind::AllocationFailed))?;
    bytes.extend(name.canonical_bytes());
    let index = builder.field_names.len();
    builder.field_names.push(bytes);
    Ok(index)
}
