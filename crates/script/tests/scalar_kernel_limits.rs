// SPDX-License-Identifier: AGPL-3.0-or-later
//! Nested verifier/validation admission cannot publish a partially compiled function.
#![allow(
    clippy::expect_used,
    reason = "Tests own valid source and independently scoped exact kernel limits."
)]

use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue, KernelVerificationBudget,
};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::scalar_compile_support;

#[test]
fn compiler_preserves_independent_kernel_admission_and_failed_work() -> Result<(), String> {
    let text = b"main(){return 1+2;}";
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    let baseline = scalar_compile_support::compile(text, ScalarCompilationBudget::default())
        .expect("Baseline");
    let usage = baseline.usage();
    let proof = baseline.program().usage();
    let exact = ScalarCompilationBudget {
        verification: KernelVerificationBudget {
            max_instructions: proof.instructions,
            max_steps: proof.verification_steps,
            max_stack_values: proof.max_stack_values,
            max_metadata_bytes: proof.verification_metadata_bytes,
        },
        validation: KernelExecutionBudget {
            max_steps: usage.validation_steps,
            max_stack_values: proof.max_stack_values,
            max_stack_bytes: proof.max_stack_values * size_of::<KernelValue>() as u64,
        },
        ..ScalarCompilationBudget::default()
    };
    assert_eq!(
        scalar_compile_support::compile(text, exact)
            .expect("Exact nested limits")
            .usage(),
        usage
    );
    for (resource, budget) in limits(exact) {
        let error =
            scalar_compile_support::compile(text, budget).expect_err("Nested resource failure");
        let ScalarCompileErrorKind::Kernel(kernel) = error.kind else {
            return Err(format!("Expected kernel limit, got {:?}", error.kind));
        };
        assert!(
            matches!(kernel.kind, KernelErrorKind::BudgetExceeded { resource: actual, .. } if actual == resource)
        );
        assert_eq!(error.work_steps, usage.compiler_steps);
        assert_eq!(error.peak_metadata_bytes, usage.peak_metadata_bytes);
        assert!(error.span.start <= error.span.end && error.span.end <= text.len() as u64);
        if resource == KernelResource::ExecutionSteps {
            assert_eq!(kernel.work_steps, usage.validation_steps - 1);
            assert_eq!(source.span(error.span).expect("Return span"), b"return 1+2");
        }
    }
    Ok(())
}

fn limits(exact: ScalarCompilationBudget) -> [(KernelResource, ScalarCompilationBudget); 7] {
    [
        (
            KernelResource::Instructions,
            ScalarCompilationBudget {
                verification: KernelVerificationBudget {
                    max_instructions: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::VerificationSteps,
            ScalarCompilationBudget {
                verification: KernelVerificationBudget {
                    max_steps: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::VerificationMetadataBytes,
            ScalarCompilationBudget {
                verification: KernelVerificationBudget {
                    max_metadata_bytes: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::StackValues,
            ScalarCompilationBudget {
                verification: KernelVerificationBudget {
                    max_stack_values: 0,
                    ..exact.verification
                },
                ..exact
            },
        ),
        (
            KernelResource::ExecutionSteps,
            ScalarCompilationBudget {
                validation: KernelExecutionBudget {
                    max_steps: exact.validation.max_steps - 1,
                    ..exact.validation
                },
                ..exact
            },
        ),
        (
            KernelResource::StackValues,
            ScalarCompilationBudget {
                validation: KernelExecutionBudget {
                    max_stack_values: 0,
                    ..exact.validation
                },
                ..exact
            },
        ),
        (
            KernelResource::ExecutionStackBytes,
            ScalarCompilationBudget {
                validation: KernelExecutionBudget {
                    max_stack_bytes: 0,
                    ..exact.validation
                },
                ..exact
            },
        ),
    ]
}
