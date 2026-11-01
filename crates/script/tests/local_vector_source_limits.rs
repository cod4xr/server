// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source triples charge fixed shape probes/tasks alongside existing nested scopes.
#![allow(
    clippy::expect_used,
    reason = "Tests compare complete compiler/invocation accounting."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue, KernelVerificationBudget,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::literal::LegacyLiteralBudget;

#[test]
fn vector_source_accepts_exact_compiler_limits_and_rejects_one_less() {
    let text = b"main(a){x=(a,2,3);return x;}";
    let usage =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Baseline vector source")
            .usage();
    assert_eq!(usage.peak_control_bytes, 0);
    let exact = FunctionCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        max_control_bytes: 0,
        validation: KernelExecutionBudget {
            max_steps: 0,
            ..KernelExecutionBudget::default()
        },
        ..FunctionCompilationBudget::default()
    };
    assert_eq!(
        function_compile_support::compile_primitive(text, exact)
            .expect("Exact source limits without pre-execution")
            .usage(),
        usage
    );
    for resource in [
        FunctionCompilationResource::Steps,
        FunctionCompilationResource::Instructions,
        FunctionCompilationResource::TaskFrames,
        FunctionCompilationResource::MetadataBytes,
        FunctionCompilationResource::LocalBindings,
        FunctionCompilationResource::IdentifierBytes,
    ] {
        let mut budget = exact;
        match resource {
            FunctionCompilationResource::Steps => budget.max_steps -= 1,
            FunctionCompilationResource::Instructions => budget.max_instructions -= 1,
            FunctionCompilationResource::TaskFrames => budget.max_task_frames -= 1,
            FunctionCompilationResource::MetadataBytes => budget.max_metadata_bytes -= 1,
            FunctionCompilationResource::LocalBindings => budget.max_local_bindings -= 1,
            FunctionCompilationResource::IdentifierBytes => budget.max_identifier_bytes -= 1,
            FunctionCompilationResource::ControlBytes => budget.max_control_bytes = 0,
        }
        let error = function_compile_support::compile_primitive(text, budget)
            .expect_err("One less compiler resource");
        assert!(
            matches!(error.kind, FunctionCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
    }
}

#[test]
fn literal_conversion_verification_and_runtime_stack_admission_remain_separate() {
    let text = b"main(a){x=(a,2,3);return x;}";
    let error = function_compile_support::compile_primitive(
        text,
        FunctionCompilationBudget {
            literal: LegacyLiteralBudget {
                max_encoded_bytes: 0,
                ..LegacyLiteralBudget::default()
            },
            ..FunctionCompilationBudget::default()
        },
    )
    .expect_err("Literal scope");
    assert!(matches!(error.kind, FunctionCompileErrorKind::Literal(_)));
    assert!(error.work_steps > 0);
    let error = function_compile_support::compile_primitive(
        text,
        FunctionCompilationBudget {
            verification: KernelVerificationBudget {
                max_stack_values: 2,
                ..KernelVerificationBudget::default()
            },
            ..FunctionCompilationBudget::default()
        },
    )
    .expect_err("Proof stack scope");
    assert!(
        matches!(error.kind, FunctionCompileErrorKind::Kernel(kernel) if matches!(kernel.kind, KernelErrorKind::BudgetExceeded { resource: KernelResource::StackValues, .. }))
    );
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Admitted source proof");
    let args = [KernelValue::Number(LegacyNumber::Integer(1))];
    let error = compiled
        .program()
        .execute_with_arguments(
            &args,
            KernelExecutionBudget {
                max_stack_bytes: 3 * size_of::<KernelValue>() as u64 - 1,
                ..KernelExecutionBudget::default()
            },
        )
        .expect_err("Execution byte scope");
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionStackBytes,
            ..
        }
    ));
    assert_eq!(error.work_steps, 0);
    assert_eq!(args[0], KernelValue::Number(LegacyNumber::Integer(1)));
}
