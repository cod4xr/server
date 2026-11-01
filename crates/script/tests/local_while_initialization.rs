// SPDX-License-Identifier: AGPL-3.0-or-later
//! Conservative loop exits do not authorize reads through body-only assignment.
#![allow(
    clippy::expect_used,
    reason = "Tests supply syntactically valid restricted-profile failures."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn first_iteration_post_loop_and_constant_false_source_reads_are_checked() {
    for text in [
        "main(arg){while(arg){PrivateLocal=PrivateLocal+1;}}",
        "main(arg){while(arg){PrivateLocal=1;arg=0;}return PrivateLocal;}",
        "main(){while(true){PrivateLocal=1;return PrivateLocal;}return PrivateLocal;}",
        "main(arg){while(arg){if(arg)PrivateLocal=1;return PrivateLocal;}}",
        "main(){while(false){return PrivateLocal;}}",
    ] {
        let error = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Uninitialized read");
        assert_eq!(error.kind, FunctionCompileErrorKind::UninitializedLocal);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source.span(error.span.clone()).expect("Name location"),
            b"PrivateLocal"
        );
        assert!(!format!("{error:?} {error}").contains("Private"));
        assert!(error.work_steps > 0);
    }
}

#[test]
fn loop_slots_remain_stable_but_later_assignment_must_initialize_them_again() {
    let compiled = function_compile_support::compile_primitive(
        b"main(arg){x=3;while(arg){y=x;arg=0;}Y=9;return x+y;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Post-loop ordinary assignment initializes the existing slot");
    assert_eq!(compiled.usage().local_bindings, 3);
    for arg in [0, 1] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Fresh loop slots")
                .value,
            KernelValue::Number(LegacyNumber::Integer(12))
        );
    }
    assert_eq!(
        function_compile_support::compile_primitive(
            b"main(){while(false){PrivateCall();}}",
            FunctionCompilationBudget::default()
        )
        .expect_err("Unsupported statement remains explicit")
        .kind,
        FunctionCompileErrorKind::UnsupportedBody
    );
}
