// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_frame::KernelFrame;

pub(super) struct KernelCallFrame {
    pub(super) function: usize,
    pub(super) machine: KernelFrame,
}
