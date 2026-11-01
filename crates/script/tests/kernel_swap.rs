// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests verify the stack permutation used to retain postfix receivers."
)]
use crate::kernel_call_support::integer;
use crate::object_heap_support::number;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelResource,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn swap_preserves_height_reverses_operands_and_uses_exact_prepaid_stack_and_fuel() {
    let program = VerifiedKernelProgram::verify(
        &[
            integer(1),
            integer(2),
            KernelInstruction::Swap,
            KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Stack permutation");
    assert_eq!(program.usage().max_stack_values, 2);
    let exact = KernelExecutionBudget {
        max_steps: 5,
        max_stack_values: 2,
        max_stack_bytes: 2 * size_of::<cod4x_script::execution::KernelValue>() as u64,
        ..KernelExecutionBudget::default()
    };
    assert_eq!(
        program.execute(exact).expect("Swapped subtraction").value,
        number(1)
    );
    for (budget, expected) in [
        (
            KernelExecutionBudget {
                max_steps: 4,
                ..exact
            },
            KernelResource::ExecutionSteps,
        ),
        (
            KernelExecutionBudget {
                max_stack_values: 1,
                ..exact
            },
            KernelResource::StackValues,
        ),
        (
            KernelExecutionBudget {
                max_stack_bytes: exact.max_stack_bytes - 1,
                ..exact
            },
            KernelResource::ExecutionStackBytes,
        ),
    ] {
        assert!(
            matches!(program.execute(budget).expect_err("One less resource").kind, KernelErrorKind::BudgetExceeded {resource, requested, maximum} if resource == expected && requested > maximum)
        );
    }
}

#[test]
fn swap_rejects_reachable_underflow_before_execution() {
    for code in [
        vec![KernelInstruction::Swap, KernelInstruction::Return],
        vec![
            integer(1),
            KernelInstruction::Swap,
            KernelInstruction::Return,
        ],
    ] {
        assert!(
            matches!(VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default()).expect_err("Two required stack values").kind, KernelErrorKind::StackUnderflow {required: 2, available} if available < 2)
        );
    }
}
