// SPDX-License-Identifier: AGPL-3.0-or-later
//! Nested verifier/validation admission cannot publish a partially compiled function.
#![allow(
    clippy::expect_used,
    reason = "Tests own valid source and independently scoped exact kernel limits."
)]

use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue, KernelVerificationBudget,
};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn compiler_preserves_independent_kernel_admission_and_failed_work() -> Result<(), String> {
    let text = b"main(){return 1+2;}";
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    let baseline =
        function_compile_support::compile_scalar(text, FunctionCompilationBudget::default())
            .expect("Baseline");
    let usage = baseline.usage();
    let proof = baseline.program().usage();
    let exact = FunctionCompilationBudget {
        verification: KernelVerificationBudget {
            max_instructions: proof.instructions,
            max_steps: proof.verification_steps,
            max_stack_values: proof.max_stack_values,
            max_metadata_bytes: proof.verification_metadata_bytes,
            max_local_values: 0,
        },
        validation: KernelExecutionBudget {
            max_steps: usage.validation_steps.expect("Pure validation"),
            max_stack_values: proof.max_stack_values,
            max_stack_bytes: proof.max_stack_values * size_of::<KernelValue>() as u64,
            max_local_values: 0,
            max_local_bytes: 0,
        },
        ..FunctionCompilationBudget::default()
    };
    assert_eq!(
        function_compile_support::compile_scalar(text, exact)
            .expect("Exact nested limits")
            .usage(),
        usage
    );
    for (resource, budget) in limits(exact) {
        let error = function_compile_support::compile_scalar(text, budget)
            .expect_err("Nested resource failure");
        let FunctionCompileErrorKind::Kernel(kernel) = error.kind else {
            return Err(format!("Expected kernel limit, got {:?}", error.kind));
        };
        assert!(
            matches!(kernel.kind, KernelErrorKind::BudgetExceeded { resource: actual, .. } if actual == resource)
        );
        assert_eq!(error.work_steps, usage.compiler_steps);
        assert_eq!(error.peak_metadata_bytes, usage.peak_metadata_bytes);
        assert!(error.span.start <= error.span.end && error.span.end <= text.len() as u64);
        if resource == KernelResource::ExecutionSteps {
            assert_eq!(
                kernel.work_steps,
                usage.validation_steps.expect("Pure validation") - 1
            );
            assert_eq!(source.span(error.span).expect("Return span"), b"return 1+2");
        }
    }
    Ok(())
}

fn limits(exact: FunctionCompilationBudget) -> [(KernelResource, FunctionCompilationBudget); 7] {
    [
        (
            KernelResource::Instructions,
            FunctionCompilationBudget {
                verification: KernelVerificationBudget {
                    max_instructions: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::VerificationSteps,
            FunctionCompilationBudget {
                verification: KernelVerificationBudget {
                    max_steps: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::VerificationMetadataBytes,
            FunctionCompilationBudget {
                verification: KernelVerificationBudget {
                    max_metadata_bytes: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::StackValues,
            FunctionCompilationBudget {
                verification: KernelVerificationBudget {
                    max_stack_values: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::ExecutionSteps,
            FunctionCompilationBudget {
                validation: KernelExecutionBudget {
                    max_steps: exact.validation.max_steps - 1,
                    ..exact.validation
                },
                ..exact
            },
        ),
        (
            KernelResource::StackValues,
            FunctionCompilationBudget {
                validation: KernelExecutionBudget {
                    max_stack_values: 0,
                    ..exact.validation
                },
                ..exact
            },
        ),
        (
            KernelResource::ExecutionStackBytes,
            FunctionCompilationBudget {
                validation: KernelExecutionBudget {
                    max_stack_bytes: 0,
                    ..exact.validation
                },
                ..exact
            },
        ),
    ]
}
