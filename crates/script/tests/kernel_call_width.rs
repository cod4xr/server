// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "The synthetic argument prefix deliberately exceeds native short namespaces."
)]

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{KernelCallBudget, KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn seventy_thousand_call_arguments_keep_wide_ordered_slots_and_missing_suffixes() {
    let mut root: Vec<_> = (0..70_000).rev().map(integer).collect();
    root.extend([
        KernelInstruction::Call {
            function: 1,
            arguments: 70_000,
        },
        KernelInstruction::Return,
    ]);
    let callee = [
        KernelInstruction::LoadLocal(69_999),
        KernelInstruction::Return,
    ];
    let missing = [
        KernelInstruction::Call {
            function: 1,
            arguments: 0,
        },
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[
        definition(&root, 0, 0),
        definition(&callee, 70_000, 70_000),
        definition(&missing, 0, 0),
    ]);
    let outcome = module
        .execute(0, &[], KernelCallBudget::default())
        .expect("Wide argument prefix copied directly into private locals");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(69_999))
    );
    assert_eq!(outcome.usage.steps, 70_004);
    assert_eq!(outcome.usage.peak_local_values, 70_000);
    assert_eq!(outcome.usage.peak_stack_values, 70_001);
    assert_eq!(outcome.usage.setup_work, 140_003);
    let outcome = module
        .execute(2, &[], KernelCallBudget::default())
        .expect("Wide missing suffix is initialized without a padded argument buffer");
    assert_eq!(outcome.value, KernelValue::Undefined);
    assert_eq!(outcome.usage.peak_local_values, 70_000);
    assert_eq!(outcome.usage.steps, 4);
    assert_eq!(outcome.usage.setup_work, 70_003);
}
