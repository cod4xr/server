// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests assemble bounded recursive call inventories and exact resource limits."
)]

use crate::kernel_call_support::{self, countdown, definition};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelInstruction, KernelResource, KernelValue,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn recursive_and_mutual_calls_run_beyond_the_archival_frame_capacity() {
    let code = countdown(0, 0);
    let module = kernel_call_support::verify(&[definition(&code, 1, 1)]);
    let outcome = module
        .execute(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(128))],
            KernelCallBudget::default(),
        )
        .expect("129 owned frames");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(128))
    );
    assert_eq!(outcome.usage.peak_frames, 129);
    assert_eq!(outcome.usage.peak_stack_values, 258);
    assert_eq!(outcome.usage.peak_local_values, 129);
    assert_eq!(outcome.usage.setup_work, 8643);
    let a = countdown(1, 3);
    let b = countdown(0, 4);
    let module = kernel_call_support::verify(&[definition(&a, 1, 1), definition(&b, 1, 1)]);
    assert_eq!(
        module
            .execute(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(31))],
                KernelCallBudget::default()
            )
            .expect("Mutual recursion")
            .value,
        KernelValue::Number(LegacyNumber::Integer(35))
    );
}

#[test]
fn shallow_call_loops_share_global_instruction_and_setup_budgets() {
    let root = [
        KernelInstruction::Call {
            function: 1,
            arguments: 0,
        },
        KernelInstruction::Discard,
        KernelInstruction::Jump(0),
    ];
    let leaf = [
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[definition(&root, 0, 0), definition(&leaf, 0, 0)]);
    let error = module
        .execute(
            0,
            &[],
            KernelCallBudget {
                max_steps: 100,
                ..KernelCallBudget::default()
            },
        )
        .expect_err("Fuel does not reset at calls");
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            requested: 101,
            maximum: 100
        }
    ));
    assert_eq!(error.cause.work_steps, 100);
    assert_eq!(error.peak_frames, 2);
    assert_eq!(error.setup_work, 41);
    let error = module
        .execute(
            0,
            &[],
            KernelCallBudget {
                max_setup_work: 20,
                ..KernelCallBudget::default()
            },
        )
        .expect_err("Repeated shallow setup stays bounded");
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::CallSetupWork,
            requested: 21,
            maximum: 20
        }
    ));
    assert_eq!(error.cause.work_steps, 46);
    assert_eq!(error.cause.instruction, Some(0));
    assert_eq!(error.setup_work, 19);
}

#[test]
fn deep_recursive_completion_and_failed_unwind_never_recurse_on_the_host_stack() {
    let code = countdown(0, 0);
    let module = kernel_call_support::verify(&[definition(&code, 1, 1)]);
    let arguments = [KernelValue::Number(LegacyNumber::Integer(12_000))];
    let budget = KernelCallBudget {
        max_setup_work: 100_000_000,
        ..KernelCallBudget::default()
    };
    let outcome = module
        .execute(0, &arguments, budget)
        .expect("12,001 flat frames");
    assert_eq!(outcome.usage.peak_frames, 12_001);
    assert_eq!(outcome.value, arguments[0]);
    let error = module
        .execute(
            0,
            &arguments,
            KernelCallBudget {
                max_frames: 12_000,
                ..budget
            },
        )
        .expect_err("Deep bounded failure drops all private frames");
    assert_eq!(error.peak_frames, 12_000);
    assert_eq!(error.cause.instruction, Some(7));
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::Frames,
            ..
        }
    ));
    assert_eq!(
        module
            .execute(0, &[KernelValue::Number(LegacyNumber::Integer(1))], budget)
            .expect("Fresh invocation after failed deep unwind")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
}
