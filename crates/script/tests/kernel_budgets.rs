// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete verifier/runtime admission, large stacks, and fuel-accounted loops.
#![allow(
    clippy::expect_used,
    reason = "Tests own finite programs and exact successful accounting."
)]

use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelResource, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::LegacyNumber;

use crate::kernel_support;

#[test]
fn verifier_exact_and_one_less_budgets_identify_each_complete_resource() {
    let input = [kernel_support::integer(7), KernelInstruction::Return];
    let program = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
        .expect("Baseline proof");
    let usage = program.usage();
    let exact = KernelVerificationBudget {
        max_instructions: usage.instructions,
        max_steps: usage.verification_steps,
        max_stack_values: usage.max_stack_values,
        max_metadata_bytes: usage.verification_metadata_bytes,
        max_local_values: 0,
    };
    assert_eq!(
        VerifiedKernelProgram::verify(&input, exact)
            .expect("Exact proof")
            .usage(),
        usage
    );
    for (resource, requested, budget) in [
        (
            KernelResource::Instructions,
            usage.instructions,
            KernelVerificationBudget {
                max_instructions: usage.instructions - 1,
                ..exact
            },
        ),
        (
            KernelResource::VerificationSteps,
            usage.verification_steps,
            KernelVerificationBudget {
                max_steps: usage.verification_steps - 1,
                ..exact
            },
        ),
        (
            KernelResource::StackValues,
            usage.max_stack_values,
            KernelVerificationBudget {
                max_stack_values: usage.max_stack_values - 1,
                ..exact
            },
        ),
        (
            KernelResource::VerificationMetadataBytes,
            usage.verification_metadata_bytes,
            KernelVerificationBudget {
                max_metadata_bytes: usage.verification_metadata_bytes - 1,
                ..exact
            },
        ),
    ] {
        let error = VerifiedKernelProgram::verify(&input, budget).expect_err("One less");
        assert_eq!(
            error.kind,
            KernelErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum: requested - 1
            }
        );
        assert!(error.work_steps <= usage.verification_steps);
        assert!(error.peak_stack_values <= usage.max_stack_values);
    }
    let zero = VerifiedKernelProgram::verify(
        &input,
        KernelVerificationBudget {
            max_steps: 0,
            ..exact
        },
    )
    .expect_err("Zero verifier work");
    assert_eq!(zero.work_steps, 0);
    assert_eq!(
        zero.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::VerificationSteps,
            requested: 1,
            maximum: 0
        }
    );
}

#[test]
fn four_thousand_stack_values_execute_with_exact_limits_and_fail_without_partial_results() {
    let count = 4096;
    let mut input = vec![kernel_support::integer(7); count];
    input.extend(std::iter::repeat_n(KernelInstruction::Discard, count - 1));
    input.push(KernelInstruction::Return);
    let program = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
        .expect("Wide stack proof");
    assert_eq!(program.usage().max_stack_values, count as u64);
    let exact = KernelExecutionBudget {
        max_steps: input.len() as u64,
        max_stack_values: count as u64,
        max_stack_bytes: count as u64 * size_of::<KernelValue>() as u64,
        max_local_values: 0,
        max_local_bytes: 0,
    };
    let outcome = program.execute(exact).expect("Complete execution");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(7)));
    assert_eq!(outcome.steps, exact.max_steps);
    assert_eq!(outcome.peak_stack_values, count as u64);
    for (resource, requested, budget) in [
        (
            KernelResource::StackValues,
            exact.max_stack_values,
            KernelExecutionBudget {
                max_stack_values: exact.max_stack_values - 1,
                ..exact
            },
        ),
        (
            KernelResource::ExecutionStackBytes,
            exact.max_stack_bytes,
            KernelExecutionBudget {
                max_stack_bytes: exact.max_stack_bytes - 1,
                ..exact
            },
        ),
        (
            KernelResource::ExecutionSteps,
            exact.max_steps,
            KernelExecutionBudget {
                max_steps: exact.max_steps - 1,
                ..exact
            },
        ),
    ] {
        let error = program
            .execute(budget)
            .expect_err("One less runtime admission");
        assert_eq!(
            error.kind,
            KernelErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum: requested - 1
            }
        );
        if resource == KernelResource::ExecutionSteps {
            assert_eq!(error.work_steps, exact.max_steps - 1);
            assert_eq!(error.peak_stack_values, count as u64);
        } else {
            assert_eq!((error.work_steps, error.peak_stack_values), (0, 0));
        }
    }
    assert_eq!(program.execute(exact).expect("Independent repeat"), outcome);
    let zero = program
        .execute(KernelExecutionBudget {
            max_steps: 0,
            max_stack_values: 0,
            max_stack_bytes: 0,
            max_local_values: 0,
            max_local_bytes: 0,
        })
        .expect_err("Fuel before stack allocation");
    assert_eq!(
        zero.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            requested: 1,
            maximum: 0
        }
    );
    assert_eq!((zero.work_steps, zero.peak_stack_values), (0, 0));
}

#[test]
fn balanced_loops_require_no_return_but_cannot_escape_per_invocation_fuel() {
    let program = VerifiedKernelProgram::verify(
        &[KernelInstruction::Jump(0)],
        KernelVerificationBudget::default(),
    )
    .expect("Balanced loop proof");
    assert_eq!(program.usage().max_stack_values, 0);
    assert_eq!(program.usage().verification_steps, 3);
    for steps in [0, 1, 17, 1024] {
        let error = program
            .execute(KernelExecutionBudget {
                max_steps: steps,
                max_stack_values: 0,
                max_stack_bytes: 0,
                max_local_values: 0,
                max_local_bytes: 0,
            })
            .expect_err("Loop fuel exhausted");
        assert_eq!(
            error.kind,
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::ExecutionSteps,
                requested: steps + 1,
                maximum: steps
            }
        );
        assert_eq!(
            (error.instruction, error.work_steps, error.peak_stack_values),
            (Some(0), steps, 0)
        );
    }
}
