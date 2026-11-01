// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;

use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

/// Failed bound-module compilation, without names/source contents or partial code.
#[derive(Clone, Eq, PartialEq)]
pub struct ModuleCompileError {
    /// Located fixed cause with global compiler work/peak metadata accounting.
    pub cause: FunctionCompileError,
    // Niche encoding keeps errors small without allocating boxes on failure.
    declaration: Option<NonZeroU64>,
}

impl ModuleCompileError {
    /// Returns the original zero-based function declaration index, if located.
    #[must_use]
    pub fn declaration_index(&self) -> Option<u64> {
        self.declaration.map(|index| index.get() - 1)
    }

    pub(super) fn at(declaration: Option<usize>, mut cause: FunctionCompileError) -> Self {
        let encoded = declaration
            .and_then(|index| (index as u64).checked_add(1))
            .and_then(NonZeroU64::new);
        if declaration.is_some() && encoded.is_none() {
            cause.kind = FunctionCompileErrorKind::SizeOverflow;
        }
        Self {
            declaration: encoded,
            cause,
        }
    }
}

impl fmt::Debug for ModuleCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModuleCompileError")
            .field("declaration_index", &self.declaration_index())
            .field("cause", &self.cause)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for ModuleCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "bound module compilation error at declaration {:?}: {}",
            self.declaration_index(),
            self.cause
        )
    }
}

impl Error for ModuleCompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
