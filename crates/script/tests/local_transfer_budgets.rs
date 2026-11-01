// SPDX-License-Identifier: AGPL-3.0-or-later
//! Linked break fixups share complete compiler admission without a second array.
#![allow(
    clippy::expect_used,
    reason = "Tests use finite source with complete baseline accounting."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompileErrorKind,
};

#[test]
fn linked_fixups_and_loop_context_frames_retain_exact_compiler_admission() {
    let text = b"main(arg){while(arg){if(arg==1)break;if(arg==2)break;arg--;continue;}return arg;}";
    let usage = scalar_compile_support::compile_local(text, ScalarCompilationBudget::default())
        .expect("Baseline linked fixups")
        .usage();
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
            .expect("Exact linked-fixup admission")
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
        let error =
            scalar_compile_support::compile_local(text, budget).expect_err("One less resource");
        assert!(
            matches!(error.kind, ScalarCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
    }
}
