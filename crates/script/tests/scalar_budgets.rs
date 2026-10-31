// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compiler admission checks preserve exact limits and work on failure.
#![allow(
    clippy::expect_used,
    reason = "Tests use valid synthetic modules and exact successful accounting."
)]

use cod4x_script::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompileErrorKind,
};
use cod4x_script::literal::{LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::scalar_compile_support;

#[test]
fn exact_and_one_less_compiler_limits_cover_work_instructions_live_tasks_and_metadata() {
    let text = b"main(){return (2+3)*4 || false;}";
    let baseline = scalar_compile_support::compile(text, ScalarCompilationBudget::default())
        .expect("Baseline compilation");
    let usage = baseline.usage();
    let exact = ScalarCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..ScalarCompilationBudget::default()
    };
    assert_eq!(
        scalar_compile_support::compile(text, exact)
            .expect("Exact limits")
            .usage(),
        usage
    );
    for (resource, budget) in [
        (
            ScalarCompilationResource::Steps,
            ScalarCompilationBudget {
                max_steps: exact.max_steps - 1,
                ..exact
            },
        ),
        (
            ScalarCompilationResource::Instructions,
            ScalarCompilationBudget {
                max_instructions: exact.max_instructions - 1,
                ..exact
            },
        ),
        (
            ScalarCompilationResource::TaskFrames,
            ScalarCompilationBudget {
                max_task_frames: exact.max_task_frames - 1,
                ..exact
            },
        ),
        (
            ScalarCompilationResource::MetadataBytes,
            ScalarCompilationBudget {
                max_metadata_bytes: exact.max_metadata_bytes - 1,
                ..exact
            },
        ),
    ] {
        let error =
            scalar_compile_support::compile(text, budget).expect_err("One less compiler limit");
        assert!(
            matches!(error.kind, ScalarCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum }
            if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0);
        assert!(error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
        assert!(error.span.end <= text.len() as u64);
    }
    let zero = scalar_compile_support::compile(
        text,
        ScalarCompilationBudget {
            max_steps: 0,
            ..exact
        },
    )
    .expect_err("No compiler fuel");
    assert_eq!(zero.work_steps, 0);
    assert_eq!(zero.peak_metadata_bytes, 0);
    assert_eq!(
        zero.kind,
        ScalarCompileErrorKind::BudgetExceeded {
            resource: ScalarCompilationResource::Steps,
            requested: 1,
            maximum: 0,
        }
    );
}

#[test]
fn numeric_literal_limits_remain_separate_from_compiler_work() {
    let text = b"main(){return -123;}";
    let exact = ScalarCompilationBudget {
        literal: LegacyLiteralBudget {
            max_encoded_bytes: 3,
            ..LegacyLiteralBudget::default()
        },
        ..ScalarCompilationBudget::default()
    };
    scalar_compile_support::compile(text, exact).expect("Exact magnitude bytes");
    let error = scalar_compile_support::compile(
        text,
        ScalarCompilationBudget {
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
        ScalarCompileErrorKind::Literal(LegacyLiteralError::BudgetExceeded(
            LegacyLiteralResource::EncodedBytes
        ))
    );
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(source.span(error.span).expect("Magnitude span"), b"123");
    assert!(error.work_steps >= 3);
}
