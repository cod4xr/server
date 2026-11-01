// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::KernelErrorKind;
use crate::execution::kernel_module_identity::KernelModuleIdentity;

/// Opaque copied reference to a function in one immutable module instance.
///
/// This carries no native pointer, serialized ABI or host authority and does not
/// keep the module alive. Indirect invocation validates the owning module; stale/
/// foreign references cannot resolve in a newly created module with reused indices.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct KernelFunctionReference {
    module: KernelModuleIdentity,
    function: usize,
}

impl KernelFunctionReference {
    pub(super) fn new(module: KernelModuleIdentity, function: usize) -> Self {
        Self { module, function }
    }

    pub(super) fn resolve(self, module: KernelModuleIdentity) -> Result<usize, KernelErrorKind> {
        if self.module != module {
            return Err(KernelErrorKind::ForeignFunctionReference);
        }
        Ok(self.function)
    }
}

impl fmt::Debug for KernelFunctionReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KernelFunctionReference(..)")
    }
}
