// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-local compilation admits binding payload/name visits alongside existing compiler scopes.
#![allow(
    clippy::expect_used,
    reason = "Tests use valid finite source and exact complete accounting."
)]

use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};
use cod4x_script::execution::{KernelErrorKind, KernelResource, KernelVerificationBudget};
use cod4x_script::statement::{StatementListBudget, StatementListErrorKind, StatementListResource};

use crate::function_compile_support;

#[test]
fn exact_and_one_less_compiler_limits_cover_local_names_payload_and_all_lowering_scopes() {
    let text = b"main(arg){x=arg+1;y=x*2;return y;}";
    let usage =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Baseline")
            .usage();
    assert_eq!(usage.local_bindings, 3);
    assert!(usage.identifier_bytes > 0);
    let exact = FunctionCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        ..FunctionCompilationBudget::default()
    };
    assert_eq!(
        function_compile_support::compile_primitive(text, exact)
            .expect("Exact limits")
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
        let error =
            function_compile_support::compile_primitive(text, budget).expect_err("One less");
        assert!(
            matches!(error.kind, FunctionCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
        assert!(error.span.end <= text.len() as u64);
    }
    let zero = function_compile_support::compile_primitive(
        text,
        FunctionCompilationBudget {
            max_steps: 0,
            ..exact
        },
    )
    .expect_err("Zero compiler work");
    assert_eq!((zero.work_steps, zero.peak_metadata_bytes), (0, 0));
}

#[test]
fn body_and_kernel_local_admission_remain_separate_sanitized_nested_scopes() -> Result<(), String> {
    let text = b"main(arg){x=arg+1;y=x*2;return y;}";
    for (resource, body) in [
        (
            StatementListResource::Statements,
            StatementListBudget {
                max_statements: 0,
                ..StatementListBudget::default()
            },
        ),
        (
            StatementListResource::Steps,
            StatementListBudget {
                max_steps: 0,
                ..StatementListBudget::default()
            },
        ),
        (
            StatementListResource::MetadataBytes,
            StatementListBudget {
                max_metadata_bytes: 0,
                ..StatementListBudget::default()
            },
        ),
    ] {
        let error = function_compile_support::compile_primitive(
            text,
            FunctionCompilationBudget {
                body,
                ..FunctionCompilationBudget::default()
            },
        )
        .expect_err("Body scope");
        let FunctionCompileErrorKind::BodyList(body) = error.kind else {
            return Err(format!("Expected body admission, got {:?}", error.kind));
        };
        assert!(
            matches!(body.kind, StatementListErrorKind::BudgetExceeded { resource: actual, .. } if actual == resource)
        );
        assert!(error.work_steps > 0);
    }
    let error = function_compile_support::compile_primitive(
        text,
        FunctionCompilationBudget {
            verification: KernelVerificationBudget {
                max_local_values: 2,
                ..KernelVerificationBudget::default()
            },
            ..FunctionCompilationBudget::default()
        },
    )
    .expect_err("Separate kernel layout admission");
    assert!(
        matches!(error.kind, FunctionCompileErrorKind::Kernel(kernel) if kernel.kind == KernelErrorKind::BudgetExceeded { resource: KernelResource::LocalValues, requested: 3, maximum: 2 })
    );
    assert!(error.work_steps > 0);
    Ok(())
}
