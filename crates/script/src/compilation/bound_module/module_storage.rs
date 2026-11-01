// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

pub(super) fn reserve<T>(count: u64, span: Range<u64>) -> Result<Vec<T>, FunctionCompileError> {
    let count = usize::try_from(count).map_err(|_| {
        function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
    })?;
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| {
        function_compile_error::at(span, FunctionCompileErrorKind::AllocationFailed)
    })?;
    Ok(values)
}
