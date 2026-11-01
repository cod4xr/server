// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;

use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

/// Failed package compilation with numeric module/local-declaration locations.
/// Names/source bytes never enter diagnostics and no partial image escapes.
#[derive(Clone, Eq, PartialEq)]
pub struct ScriptPackageError {
    /// Located fixed cause with shared compiler work/metadata accounting.
    pub cause: FunctionCompileError,
    module: Option<NonZeroU64>,
    declaration: Option<NonZeroU64>,
}

impl ScriptPackageError {
    /// Returns the input-order module index, if the failure is located.
    #[must_use]
    pub fn module_index(&self) -> Option<u64> {
        self.module.map(|index| index.get() - 1)
    }

    /// Returns the original declaration index within that module, if located.
    #[must_use]
    pub fn declaration_index(&self) -> Option<u64> {
        self.declaration.map(|index| index.get() - 1)
    }

    pub(super) fn at(
        module: Option<usize>,
        declaration: Option<u64>,
        mut cause: FunctionCompileError,
    ) -> Self {
        let encoded_module = module
            .and_then(|index| (index as u64).checked_add(1))
            .and_then(NonZeroU64::new);
        let encoded_declaration = declaration
            .and_then(|index| index.checked_add(1))
            .and_then(NonZeroU64::new);
        if (module.is_some() && encoded_module.is_none())
            || (declaration.is_some() && encoded_declaration.is_none())
        {
            cause.kind = FunctionCompileErrorKind::SizeOverflow;
        }
        Self {
            cause,
            module: encoded_module,
            declaration: encoded_declaration,
        }
    }
}

impl fmt::Debug for ScriptPackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScriptPackageError")
            .field("module", &self.module_index())
            .field("declaration", &self.declaration_index())
            .field("cause", &self.cause)
            .finish_non_exhaustive()
    }
}
impl fmt::Display for ScriptPackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "script package error at module {:?}, declaration {:?}: {}",
            self.module_index(),
            self.declaration_index(),
            self.cause
        )
    }
}
impl Error for ScriptPackageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
