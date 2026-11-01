// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::binding::LegacyIdentifier;
use crate::compilation::bound_module::staged_function::StagedFunctionCode;
use crate::compilation::bound_module::{CompiledModuleDeclaration, CompiledModuleFunction};
use crate::compilation::function_call_scope::FunctionCallTarget;
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::{KernelFunctionDefinition, KernelInstruction};

pub(super) fn base(
    declarations: u64,
    functions: u64,
    span: Range<u64>,
) -> Result<u64, FunctionCompileError> {
    let active_size = size_of::<(LegacyIdentifier<'_>, FunctionCallTarget)>()
        + size_of::<CompiledModuleFunction>()
        + size_of::<StagedFunctionCode>()
        + size_of::<KernelFunctionDefinition<'_>>();
    add(
        bytes(
            declarations,
            size_of::<CompiledModuleDeclaration>(),
            span.clone(),
        )?,
        bytes(functions, active_size, span.clone())?,
        span,
    )
}

pub(super) fn staged(instructions: u64, span: Range<u64>) -> Result<u64, FunctionCompileError> {
    bytes(
        instructions,
        size_of::<KernelInstruction>() + size_of::<Range<u64>>(),
        span,
    )
}

pub(super) fn retained(
    declarations: u64,
    functions: u64,
    instructions: u64,
    span: Range<u64>,
) -> Result<u64, FunctionCompileError> {
    add(
        add(
            bytes(
                declarations,
                size_of::<CompiledModuleDeclaration>(),
                span.clone(),
            )?,
            bytes(functions, size_of::<CompiledModuleFunction>(), span.clone())?,
            span.clone(),
        )?,
        bytes(instructions, size_of::<Range<u64>>(), span.clone())?,
        span,
    )
}

fn bytes(count: u64, size: usize, span: Range<u64>) -> Result<u64, FunctionCompileError> {
    count
        .checked_mul(size as u64)
        .ok_or_else(|| function_compile_error::at(span, FunctionCompileErrorKind::SizeOverflow))
}

fn add(left: u64, right: u64, span: Range<u64>) -> Result<u64, FunctionCompileError> {
    left.checked_add(right)
        .ok_or_else(|| function_compile_error::at(span, FunctionCompileErrorKind::SizeOverflow))
}
