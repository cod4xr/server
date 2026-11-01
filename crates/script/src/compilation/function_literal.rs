// SPDX-License-Identifier: AGPL-3.0-or-later
use std::str;

use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelValue;
use crate::expression::LegacyLiteralPolarity;
use crate::legacy::LegacyNumber;
use crate::lexical::{LegacyLexeme, LegacyLexemeKind};
use crate::literal::{
    LegacyLiteral, LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource,
};

pub(super) fn decode(
    token: LegacyLexeme<'_>,
    polarity: LegacyLiteralPolarity,
    budget: LegacyLiteralBudget,
) -> Result<KernelValue, FunctionCompileError> {
    let error =
        |kind| function_compile_error::at(token.span(), FunctionCompileErrorKind::Literal(kind));
    let LegacyLexemeKind::Token(code) = token.kind() else {
        return Err(error(LegacyLiteralError::WrongToken));
    };
    if matches!(code.as_u16(), 0x103 | 0x104) {
        return Err(function_compile_error::at(
            token.span(),
            FunctionCompileErrorKind::UnsupportedPrimary,
        ));
    }
    if token.as_bytes().len() as u64 > budget.max_encoded_bytes {
        return Err(error(LegacyLiteralError::BudgetExceeded(
            LegacyLiteralResource::EncodedBytes,
        )));
    }
    if polarity == LegacyLiteralPolarity::Negated && code.as_u16() == 0x11f {
        let text = str::from_utf8(token.as_bytes())
            .map_err(|_| error(LegacyLiteralError::InvalidSpelling))?;
        let magnitude = text
            .parse::<u64>()
            .map_err(|_| error(LegacyLiteralError::NumberOutOfRange))?;
        let value = if magnitude == 1_u64 << 31 {
            i32::MIN
        } else {
            -i32::try_from(magnitude).map_err(|_| error(LegacyLiteralError::NumberOutOfRange))?
        };
        return Ok(KernelValue::Number(LegacyNumber::Integer(value)));
    }
    let value = LegacyLiteral::decode(token, budget).map_err(error)?;
    match (value, polarity) {
        (LegacyLiteral::Undefined, LegacyLiteralPolarity::Unchanged) => Ok(KernelValue::Undefined),
        (LegacyLiteral::Number(value), LegacyLiteralPolarity::Unchanged) => {
            Ok(KernelValue::Number(value))
        }
        (LegacyLiteral::Number(LegacyNumber::Float(value)), LegacyLiteralPolarity::Negated) => {
            Ok(KernelValue::Number(LegacyNumber::Float(f32::from_bits(
                value.to_bits() ^ 0x8000_0000,
            ))))
        }
        _ => Err(function_compile_error::at(
            token.span(),
            FunctionCompileErrorKind::InvalidStructure,
        )),
    }
}
