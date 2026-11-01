// SPDX-License-Identifier: AGPL-3.0-or-later
//! Initialization is intersected only across structural paths that reach a read.
#![allow(
    clippy::expect_used,
    reason = "Tests use syntactically valid restricted-profile examples."
)]

use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};
use cod4x_script::execution::{KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};

use crate::scalar_compile_support;

#[test]
fn one_path_assignment_and_else_reads_do_not_inherit_then_facts_even_for_constants() {
    for text in [
        "main(arg){if(arg)PrivateLocal=1;return PrivateLocal;}",
        "main(){if(true)PrivateLocal=1;return PrivateLocal;}",
        "main(arg){if(arg)PrivateLocal=1;else PrivateLocal=PrivateLocal+1;return PrivateLocal;}",
        "main(arg){if(arg){PrivateLocal=1;}else{}return PrivateLocal;}",
        "main(arg){if(arg){}else PrivateLocal=1;return PrivateLocal;}",
        "main(arg){if(arg){if(arg)PrivateLocal=1;}else PrivateLocal=1;return PrivateLocal;}",
        "main(arg){if(arg){PrivateLocal=1;return PrivateLocal;}else{}return PrivateLocal;}",
    ] {
        let error = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect_err("Not initialized on every reaching path");
        assert_eq!(error.kind, ScalarCompileErrorKind::UninitializedLocal);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source.span(error.span.clone()).expect("Read location"),
            b"PrivateLocal"
        );
        assert!(error.work_steps > 0);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn stable_function_slots_preserve_incoming_values_case_identity_and_branch_reassignment() {
    let text = b"main(arg){x=5;if(arg){X=x+1;y=x;}else{x=x+2;Y=x;}return x+y;}";
    let compiled = scalar_compile_support::compile_local(text, ScalarCompilationBudget::default())
        .expect("Both paths initialize the same canonical slots");
    assert_eq!(compiled.usage().local_bindings, 3);
    for (arg, expected) in [(0, 14), (1, 12)] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Independent branch invocation")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
}

#[test]
fn known_returning_paths_reject_later_source_without_changing_immediate_return_policy() {
    for (text, kind) in [
        (
            "main(){{return 1;}x=2;}",
            ScalarCompileErrorKind::UnreachableStatement,
        ),
        (
            "main(arg){if(arg)return 1;else return 2;return 3;}",
            ScalarCompileErrorKind::UnreachableStatement,
        ),
        (
            "main(arg){if(arg){return 1;x=2;}else return 2;}",
            ScalarCompileErrorKind::ReturnNotLast,
        ),
        (
            "main(){return 1;x=2;}",
            ScalarCompileErrorKind::ReturnNotLast,
        ),
        (
            "main(){if(false){PrivateCall();}}",
            ScalarCompileErrorKind::UnsupportedBody,
        ),
    ] {
        assert_eq!(
            scalar_compile_support::compile_local(
                text.as_bytes(),
                ScalarCompilationBudget::default()
            )
            .expect_err("Explicit structural policy")
            .kind,
            kind
        );
    }
}
