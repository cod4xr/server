// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite wide references and arguments explicitly admit their logical payload."
)]

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn indirect_calls_preserve_wide_function_indices_and_argument_prefixes() {
    let mut root = Vec::with_capacity(70_003);
    for value in (0..70_000).rev() {
        root.push(integer(value));
    }
    root.extend([
        KernelInstruction::FunctionReference(65_999),
        KernelInstruction::CallIndirect { arguments: 70_000 },
        KernelInstruction::Return,
    ]);
    let leaf = [
        KernelInstruction::LoadLocal(69_999),
        KernelInstruction::Return,
    ];
    let empty = [
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let mut definitions = vec![definition(&empty, 0, 0); 66_000];
    definitions[0] = definition(&root, 0, 0);
    definitions[65_999] = definition(&leaf, 70_000, 70_000);
    let module = kernel_call_support::verify(&definitions);
    let result = module
        .execute(0, &[], KernelCallBudget::default())
        .expect("Wide indirect call");
    assert_eq!(
        result.value,
        KernelValue::Number(LegacyNumber::Integer(69_999))
    );
    assert_eq!(result.usage.steps, 70_005);
    assert_eq!(result.usage.peak_local_values, 70_000);
    let error = module
        .execute(65_998, &[result.value], KernelCallBudget::default())
        .expect_err("Independent neighboring function has zero parameters");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 0,
            actual: 1
        }
    );
    assert_eq!(
        module
            .execute(0, &[], KernelCallBudget::default())
            .expect("Fresh recovery"),
        result
    );
}
