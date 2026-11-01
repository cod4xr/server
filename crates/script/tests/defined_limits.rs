// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests use finite wide builtin programs and exact budgets."
)]
use crate::kernel_call_support::integer;
use cod4x_script::execution::{
    KernelBuiltin, KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelResource,
    KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyArithmetic, LegacyNumber};

#[test]
fn wide_definedness_discards_exactly_its_operands_under_shared_stack_and_fuel_limits() {
    let count = 65_536;
    let mut code = vec![integer(40)];
    code.extend(vec![
        KernelInstruction::Constant(KernelValue::Undefined);
        count - 1
    ]);
    code.extend([
        integer(0),
        KernelInstruction::Builtin {
            function: KernelBuiltin::IsDefined,
            arguments: count,
        },
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ]);
    let program = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect("Wide stack proof");
    let exact = KernelExecutionBudget {
        max_steps: count as u64 + 4,
        max_stack_values: count as u64 + 1,
        max_stack_bytes: (count as u64 + 1) * size_of::<KernelValue>() as u64,
        ..KernelExecutionBudget::default()
    };
    let result = program
        .execute(exact)
        .expect("Complete discard retains older prefix");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(41)));
    assert_eq!(result.steps, exact.max_steps);
    for (resource, budget) in [
        (
            KernelResource::ExecutionSteps,
            KernelExecutionBudget {
                max_steps: exact.max_steps - 1,
                ..exact
            },
        ),
        (
            KernelResource::StackValues,
            KernelExecutionBudget {
                max_stack_values: exact.max_stack_values - 1,
                ..exact
            },
        ),
        (
            KernelResource::ExecutionStackBytes,
            KernelExecutionBudget {
                max_stack_bytes: exact.max_stack_bytes - 1,
                ..exact
            },
        ),
    ] {
        assert!(
            matches!(program.execute(budget).expect_err("One less shared resource").kind,
            KernelErrorKind::BudgetExceeded { resource: actual, .. } if actual == resource)
        );
    }
}
