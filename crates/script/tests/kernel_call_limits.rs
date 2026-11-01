// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests use verified finite recursion to measure exact global admissions."
)]

use crate::kernel_call_support::{self, countdown, definition};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelResource, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn every_aggregate_execution_limit_accepts_exact_usage_and_rejects_one_less() -> Result<(), String>
{
    let code = countdown(0, 0);
    let module = kernel_call_support::verify(&[definition(&code, 1, 1)]);
    let arguments = [KernelValue::Number(LegacyNumber::Integer(64))];
    let outcome = module
        .execute(0, &arguments, KernelCallBudget::default())
        .expect("Measure logical admissions");
    let usage = outcome.usage;
    let exact = KernelCallBudget {
        max_steps: usage.steps,
        max_setup_work: usage.setup_work,
        max_frames: usage.peak_frames,
        max_frame_bytes: usage.peak_frame_bytes,
        max_stack_values: usage.peak_stack_values,
        max_stack_bytes: usage.peak_stack_bytes,
        max_local_values: usage.peak_local_values,
        max_local_bytes: usage.peak_local_bytes,
    };
    assert_eq!(
        module
            .execute(0, &arguments, exact)
            .expect("Exact aggregate resources"),
        outcome
    );
    for resource in [
        KernelResource::ExecutionSteps,
        KernelResource::CallSetupWork,
        KernelResource::Frames,
        KernelResource::ExecutionFrameBytes,
        KernelResource::StackValues,
        KernelResource::ExecutionStackBytes,
        KernelResource::LocalValues,
        KernelResource::ExecutionLocalBytes,
    ] {
        let mut budget = exact;
        match resource {
            KernelResource::ExecutionSteps => budget.max_steps -= 1,
            KernelResource::CallSetupWork => budget.max_setup_work -= 1,
            KernelResource::Frames => budget.max_frames -= 1,
            KernelResource::ExecutionFrameBytes => budget.max_frame_bytes -= 1,
            KernelResource::StackValues => budget.max_stack_values -= 1,
            KernelResource::ExecutionStackBytes => budget.max_stack_bytes -= 1,
            KernelResource::LocalValues => budget.max_local_values -= 1,
            KernelResource::ExecutionLocalBytes => budget.max_local_bytes -= 1,
            _ => return Err("Unexpected execution resource in exact-limit fixture".into()),
        }
        let error = module
            .execute(0, &arguments, budget)
            .expect_err("One less aggregate resource");
        assert!(
            matches!(error.cause.kind, KernelErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.cause.work_steps <= budget.max_steps);
        assert!(error.setup_work <= budget.max_setup_work);
        assert!(error.peak_frames <= budget.max_frames);
        assert!(error.peak_reserved_stack_values <= budget.max_stack_values);
        assert!(error.peak_local_values <= budget.max_local_values);
    }
    Ok(())
}

#[test]
fn invalid_entries_excess_arguments_and_zero_fuel_fail_without_setup() {
    let code = countdown(0, 0);
    let module = kernel_call_support::verify(&[definition(&code, 1, 1)]);
    let arguments = [KernelValue::Undefined; 2];
    for entry in [1, u64::MAX] {
        let error = module
            .execute(entry, &[], KernelCallBudget::default())
            .expect_err("Invalid entry");
        assert_eq!(error.function, None);
        assert_eq!(
            error.cause.kind,
            KernelErrorKind::InvalidFunction { function: entry }
        );
        assert_eq!(error.setup_work, 0);
        assert_eq!(error.peak_frames, 0);
    }
    let error = module
        .execute(0, &arguments, KernelCallBudget::default())
        .expect_err("Excess entry arguments");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 1,
            actual: 2
        }
    );
    assert_eq!(error.setup_work, 0);
    let error = module
        .execute(
            0,
            &[],
            KernelCallBudget {
                max_steps: 0,
                ..KernelCallBudget::default()
            },
        )
        .expect_err("No instruction admission");
    assert_eq!(error.setup_work, 0);
    assert_eq!(error.cause.work_steps, 0);
    assert_eq!(error.peak_frames, 0);
    assert_eq!(arguments, [KernelValue::Undefined; 2]);
}
