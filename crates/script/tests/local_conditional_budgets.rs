// SPDX-License-Identifier: AGPL-3.0-or-later
//! Snapshot payload and every existing lowering scope retain exact admission.
#![allow(
    clippy::expect_used,
    reason = "Tests compare complete admitted accounting for finite source."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompileErrorKind,
};

#[test]
fn snapshot_and_existing_limits_accept_exactly_and_reject_one_less() {
    let text = b"main(arg){x=1;if(arg){y=x;}else{y=2;}return y;}";
    let usage = scalar_compile_support::compile_local(text, ScalarCompilationBudget::default())
        .expect("Baseline")
        .usage();
    // Incoming arg/x plus the then snapshot arg/x/y overlap before restore.
    assert_eq!(usage.peak_control_bytes, 5);
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
            .expect("All exact limits")
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
        let error = scalar_compile_support::compile_local(text, budget).expect_err("One less");
        assert!(
            matches!(error.kind, ScalarCompileErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
        assert!(error.span.end <= text.len() as u64);
    }
}

#[test]
fn snapshots_release_between_siblings_and_zero_name_snapshots_need_no_payload() {
    let budget = ScalarCompilationBudget {
        max_control_bytes: 2,
        ..ScalarCompilationBudget::default()
    };
    let compiled = scalar_compile_support::compile_local(
        b"main(arg){if(arg){}else{}if(arg){}else{}if(arg){}else{}}",
        budget,
    )
    .expect("Sibling snapshots release their payload");
    assert_eq!(compiled.usage().peak_control_bytes, 2);
    for text in [b"main(){if(true){}else{}}".as_slice(), b"main(){}"] {
        let compiled = scalar_compile_support::compile_local(
            text,
            ScalarCompilationBudget {
                max_control_bytes: 0,
                ..budget
            },
        )
        .expect("No flag storage");
        assert_eq!(compiled.usage().peak_control_bytes, 0);
    }
    let error = scalar_compile_support::compile_local(
        b"main(arg){if(arg){if(arg){}}}",
        ScalarCompilationBudget {
            max_control_bytes: 1,
            ..budget
        },
    )
    .expect_err("Nested incoming facts overlap");
    assert!(matches!(
        error.kind,
        ScalarCompileErrorKind::BudgetExceeded {
            resource: ScalarCompilationResource::ControlBytes,
            requested: 2,
            maximum: 1
        }
    ));
}
