// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::execution::KernelError;

/// Located module proof/invocation failure, without source names or value contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelModuleError {
    /// Function index; None for whole-inventory/entry admission failures.
    pub function: Option<u64>,
    /// Fixed cause and optional instruction. Work is global for the active phase.
    pub cause: KernelError,
    /// Admitted invocation setup work; zero during verification.
    pub setup_work: u64,
    /// Peak admitted invocation frames; zero during verification.
    pub peak_frames: u64,
    /// Peak admitted aggregate reserved operands; zero during verification.
    pub peak_reserved_stack_values: u64,
    /// Peak admitted aggregate local slots; zero during verification.
    pub peak_local_values: u64,
}

impl KernelModuleError {
    pub(super) fn verification(function: Option<u64>, cause: KernelError) -> Self {
        Self {
            function,
            cause,
            setup_work: 0,
            peak_frames: 0,
            peak_reserved_stack_values: 0,
            peak_local_values: 0,
        }
    }
}

impl fmt::Display for KernelModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "primitive module error at function {:?}: {}",
            self.function, self.cause
        )
    }
}

impl Error for KernelModuleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
