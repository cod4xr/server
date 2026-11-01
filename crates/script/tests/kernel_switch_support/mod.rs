// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Typed switch fixtures require complete valid immutable proofs."
)]
use crate::kernel_call_support;
use cod4x_script::execution::{
    KernelInstruction, KernelSwitchComparison, KernelSwitchKey, VerifiedKernelModule,
};

pub fn module(
    key: KernelSwitchKey,
    comparison: KernelSwitchComparison,
    labels: &[Vec<u8>],
) -> VerifiedKernelModule {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::RequireSwitchKey,
        KernelInstruction::SwitchCompare { key, comparison },
        KernelInstruction::Return,
    ];
    let mut definition = kernel_call_support::definition(&code, 1, 1);
    definition.switch_labels = labels;
    kernel_call_support::verify(&[definition])
}
