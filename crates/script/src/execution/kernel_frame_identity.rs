// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_module_identity::KernelModuleIdentity;

/// Invocation-local owner depth and immutable image identity have separate roles.
#[derive(Clone, Copy)]
pub(super) struct KernelFrameIdentity {
    pub(super) depth: usize,
    pub(super) module: Option<KernelModuleIdentity>,
}
