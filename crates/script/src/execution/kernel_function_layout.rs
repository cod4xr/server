// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::{KernelError, KernelErrorKind};

/// Immutable scalar parameter-prefix/total-local layout, without names or native ABI.
///
/// Parameters are part of the total local count. Actual count/storage admission
/// occurs separately at verification/invocation; constructing this value allocates nothing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KernelFunctionLayout {
    pub(super) parameters: usize,
    pub(super) local_values: usize,
}

impl KernelFunctionLayout {
    /// Checks the parameter prefix and converts wide counts without truncation.
    ///
    /// # Errors
    /// Rejects parameters exceeding total locals or counts outside host address space.
    /// No storage is allocated or runtime state changed.
    pub fn new(parameters: u64, local_values: u64) -> Result<Self, KernelError> {
        if parameters > local_values {
            return Err(kernel_error::general(
                KernelErrorKind::InvalidFunctionLayout {
                    parameters,
                    local_values,
                },
            ));
        }
        Ok(Self {
            parameters: usize::try_from(parameters)
                .map_err(|_| kernel_error::general(KernelErrorKind::SizeOverflow))?,
            local_values: usize::try_from(local_values)
                .map_err(|_| kernel_error::general(KernelErrorKind::SizeOverflow))?,
        })
    }

    /// Returns declared parameter slots; invocation entry points select binding policy.
    #[must_use]
    pub fn parameters(self) -> u64 {
        self.parameters as u64
    }

    /// Returns total local slots, including the parameter prefix.
    #[must_use]
    pub fn local_values(self) -> u64 {
        self.local_values as u64
    }
}
