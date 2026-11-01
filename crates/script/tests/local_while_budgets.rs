// SPDX-License-Identifier: AGPL-3.0-or-later
//! Loop lowering and invocation retain independent exact resource admission.
#![allow(
    clippy::expect_used,
    reason = "Tests compare admitted accounting on finite source."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompileErrorKind,
};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn loop_lowering_accepts_exact_resource_limits_and_rejects_one_less() {
    let text = b"main(arg){x=0;while(arg){x++;arg--;}return x;}";
    let compiled = scalar_compile_support::compile_local(text, ScalarCompilationBudget::default())
        .expect("Baseline loop");
    let usage = compiled.usage();
    assert_eq!(usage.peak_control_bytes, 2);
    let exact = ScalarCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        max_control_bytes: usage.peak_control_bytes,
        ..ScalarCompilationBudget::default()
    };
    assert_eq!(
        scalar_compile_support::compile_local(text, exact)
            .expect("Exact compiler limits")
            .usage(),
        usage
    );
    for resource in [
        ScalarCompilationResource::Steps,
        ScalarCompilationResource::Instructions,
        ScalarCompilationResource::TaskFrames,
        ScalarCompilationResource::MetadataBytes,
        ScalarCompilationResource::LocalBindings,
        ScalarCompilationResource::IdentifierBytes,
        ScalarCompilationResource::ControlBytes,
    ] {
        let mut budget = exact;
        match resource {
            ScalarCompilationResource::Steps => budget.max_steps -= 1,
            ScalarCompilationResource::Instructions => budget.max_instructions -= 1,
            ScalarCompilationResource::TaskFrames => budget.max_task_frames -= 1,
            ScalarCompilationResource::MetadataBytes => budget.max_metadata_bytes -= 1,
            ScalarCompilationResource::LocalBindings => budget.max_local_bindings -= 1,
            ScalarCompilationResource::IdentifierBytes => budget.max_identifier_bytes -= 1,
            ScalarCompilationResource::ControlBytes => budget.max_control_bytes -= 1,
        }
        let error = scalar_compile_support::compile_local(text, budget)
            .expect_err("One less compiler resource");
        assert!(
            matches!(error.kind, ScalarCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
    }
}

#[test]
fn loop_execution_fuel_accepts_exact_completion_and_rejects_one_less() {
    let compiled = scalar_compile_support::compile_local(
        b"main(arg){while(arg){arg--;}return arg;}",
        ScalarCompilationBudget::default(),
    )
    .expect("Finite loop");
    let args = [KernelValue::Number(LegacyNumber::Integer(9))];
    let outcome = compiled
        .program()
        .execute_with_arguments(&args, KernelExecutionBudget::default())
        .expect("Baseline invocation");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(0)));
    let exact = KernelExecutionBudget {
        max_steps: outcome.steps,
        ..KernelExecutionBudget::default()
    };
    assert_eq!(
        compiled
            .program()
            .execute_with_arguments(&args, exact)
            .expect("Exact fuel"),
        outcome
    );
    let error = compiled
        .program()
        .execute_with_arguments(
            &args,
            KernelExecutionBudget {
                max_steps: outcome.steps - 1,
                ..exact
            },
        )
        .expect_err("One less fuel");
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            ..
        }
    ));
    assert_eq!(error.work_steps, outcome.steps - 1);
    assert_eq!(args[0], KernelValue::Number(LegacyNumber::Integer(9)));
}
