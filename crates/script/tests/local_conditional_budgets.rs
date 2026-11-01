// SPDX-License-Identifier: AGPL-3.0-or-later
//! Snapshot payload and every existing lowering scope retain exact admission.
#![allow(
    clippy::expect_used,
    reason = "Tests compare complete admitted accounting for finite source."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};

#[test]
fn snapshot_and_existing_limits_accept_exactly_and_reject_one_less() {
    let text = b"main(arg){x=1;if(arg){y=x;}else{y=2;}return y;}";
    let usage =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Baseline")
            .usage();
    // Incoming arg/x plus the then snapshot arg/x/y overlap before restore.
    assert_eq!(usage.peak_control_bytes, 5);
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
            .expect("All exact limits")
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
        let error =
            function_compile_support::compile_primitive(text, budget).expect_err("One less");
        assert!(
            matches!(error.kind, FunctionCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
        assert!(error.span.end <= text.len() as u64);
    }
}

#[test]
fn snapshots_release_between_siblings_and_zero_name_snapshots_need_no_payload() {
    let budget = FunctionCompilationBudget {
        max_control_bytes: 2,
        ..FunctionCompilationBudget::default()
    };
    let compiled = function_compile_support::compile_primitive(
        b"main(arg){if(arg){}else{}if(arg){}else{}if(arg){}else{}}",
        budget,
    )
    .expect("Sibling snapshots release their payload");
    assert_eq!(compiled.usage().peak_control_bytes, 2);
    for text in [b"main(){if(true){}else{}}".as_slice(), b"main(){}"] {
        let compiled = function_compile_support::compile_primitive(
            text,
            FunctionCompilationBudget {
                max_control_bytes: 0,
                ..budget
            },
        )
        .expect("No flag storage");
        assert_eq!(compiled.usage().peak_control_bytes, 0);
    }
    let error = function_compile_support::compile_primitive(
        b"main(arg){if(arg){if(arg){}}}",
        FunctionCompilationBudget {
            max_control_bytes: 1,
            ..budget
        },
    )
    .expect_err("Nested incoming facts overlap");
    assert!(matches!(
        error.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::ControlBytes,
            requested: 2,
            maximum: 1
        }
    ));
}
