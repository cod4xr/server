// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compiler admission checks preserve exact limits and work on failure.
#![allow(
    clippy::expect_used,
    reason = "Tests use valid synthetic modules and exact successful accounting."
)]

use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};
use cod4x_script::literal::{LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn exact_and_one_less_compiler_limits_cover_work_instructions_live_tasks_and_metadata() {
    let text = b"main(){return (2+3)*4 || false;}";
    let baseline =
        function_compile_support::compile_scalar(text, FunctionCompilationBudget::default())
            .expect("Baseline compilation");
    let usage = baseline.usage();
    let exact = FunctionCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..FunctionCompilationBudget::default()
    };
    assert_eq!(
        function_compile_support::compile_scalar(text, exact)
            .expect("Exact limits")
            .usage(),
        usage
    );
    for (resource, budget) in [
        (
            FunctionCompilationResource::Steps,
            FunctionCompilationBudget {
                max_steps: exact.max_steps - 1,
                ..exact
            },
        ),
        (
            FunctionCompilationResource::Instructions,
            FunctionCompilationBudget {
                max_instructions: exact.max_instructions - 1,
                ..exact
            },
        ),
        (
            FunctionCompilationResource::TaskFrames,
            FunctionCompilationBudget {
                max_task_frames: exact.max_task_frames - 1,
                ..exact
            },
        ),
        (
            FunctionCompilationResource::MetadataBytes,
            FunctionCompilationBudget {
                max_metadata_bytes: exact.max_metadata_bytes - 1,
                ..exact
            },
        ),
    ] {
        let error = function_compile_support::compile_scalar(text, budget)
            .expect_err("One less compiler limit");
        assert!(
            matches!(error.kind, FunctionCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum }
            if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0);
        assert!(error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
        assert!(error.span.end <= text.len() as u64);
    }
    let zero = function_compile_support::compile_scalar(
        text,
        FunctionCompilationBudget {
            max_steps: 0,
            ..exact
        },
    )
    .expect_err("No compiler fuel");
    assert_eq!(zero.work_steps, 0);
    assert_eq!(zero.peak_metadata_bytes, 0);
    assert_eq!(
        zero.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::Steps,
            requested: 1,
            maximum: 0,
        }
    );
}

#[test]
fn numeric_literal_limits_remain_separate_from_compiler_work() {
    let text = b"main(){return -123;}";
    let exact = FunctionCompilationBudget {
        literal: LegacyLiteralBudget {
            max_encoded_bytes: 3,
            ..LegacyLiteralBudget::default()
        },
        ..FunctionCompilationBudget::default()
    };
    function_compile_support::compile_scalar(text, exact).expect("Exact magnitude bytes");
    let error = function_compile_support::compile_scalar(
        text,
        FunctionCompilationBudget {
            literal: LegacyLiteralBudget {
                max_encoded_bytes: 2,
                ..exact.literal
            },
            ..exact
        },
    )
    .expect_err("Literal byte limit");
    assert_eq!(
        error.kind,
        FunctionCompileErrorKind::Literal(LegacyLiteralError::BudgetExceeded(
            LegacyLiteralResource::EncodedBytes
        ))
    );
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(source.span(error.span).expect("Magnitude span"), b"123");
    assert!(error.work_steps >= 3);
}
