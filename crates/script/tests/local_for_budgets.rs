// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deferred continue facts/fixups share global work and exact compiler admission.
#![allow(
    clippy::expect_used,
    reason = "Tests compare complete accounting for finite source."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};

#[test]
fn deferred_continue_snapshots_and_all_compiler_scopes_accept_exactly() {
    let text = b"main(n){for(i=0;i<n;x++){if(i%3==0){x=1;i++;continue;}if(i%3==1){x=2;i++;continue;}x=3;i++;}return i;}";
    let usage =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Baseline for joins")
            .usage();
    assert!(usage.peak_control_bytes > usage.local_bindings);
    let exact = FunctionCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        max_control_bytes: usage.peak_control_bytes,
        ..FunctionCompilationBudget::default()
    };
    assert_eq!(
        function_compile_support::compile_primitive(text, exact)
            .expect("Exact admitted for function")
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
        FunctionCompilationResource::ControlBytes,
    ] {
        let mut budget = exact;
        match resource {
            FunctionCompilationResource::Steps => budget.max_steps -= 1,
            FunctionCompilationResource::Instructions => budget.max_instructions -= 1,
            FunctionCompilationResource::TaskFrames => budget.max_task_frames -= 1,
            FunctionCompilationResource::MetadataBytes => budget.max_metadata_bytes -= 1,
            FunctionCompilationResource::LocalBindings => budget.max_local_bindings -= 1,
            FunctionCompilationResource::IdentifierBytes => budget.max_identifier_bytes -= 1,
            FunctionCompilationResource::ControlBytes => budget.max_control_bytes -= 1,
        }
        let error = function_compile_support::compile_primitive(text, budget)
            .expect_err("One less resource");
        assert!(
            matches!(error.kind, FunctionCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
    }
}
