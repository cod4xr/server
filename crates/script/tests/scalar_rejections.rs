// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unsupported source never fabricates a callable scalar implementation.
#![allow(
    clippy::expect_used,
    reason = "Tests supply syntactically valid restricted-profile failures."
)]

use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};

use crate::scalar_compile_support;

#[test]
fn unsupported_parameters_and_statement_bodies_are_rejected_explicitly() {
    for (text, kind) in [
        (
            "main(arg){return 1;}",
            ScalarCompileErrorKind::UnsupportedParameters,
        ),
        (
            "main(){return 1;return 2;}",
            ScalarCompileErrorKind::UnsupportedBody,
        ),
        ("main(){x=1;}", ScalarCompileErrorKind::UnsupportedBody),
        (
            "main(){if(true)return 1;}",
            ScalarCompileErrorKind::UnsupportedBody,
        ),
        ("main(){wait 1;}", ScalarCompileErrorKind::UnsupportedBody),
        (
            "main(){{return 1;}}",
            ScalarCompileErrorKind::UnsupportedBody,
        ),
    ] {
        let error =
            scalar_compile_support::compile(text.as_bytes(), ScalarCompilationBudget::default())
                .expect_err("Unsupported function");
        assert_eq!(error.kind, kind);
        assert!(error.work_steps > 0);
        assert!(error.span.start <= error.span.end);
        assert!(error.span.end <= text.len() as u64);
    }
}

#[test]
fn unsupported_values_are_rejected_even_in_short_circuited_source() {
    for expression in [
        "\"private-value\"",
        "&\"private-value\"",
        "self",
        "level",
        "game",
        "anim",
        "[]",
        "private_name",
        "self.private_field",
        "private_array[0]",
        "private_call()",
        "::private_function",
        "true || private_call()",
        "false && private_name",
    ] {
        let text = format!("PrivateMain(){{return {expression};}}");
        let error =
            scalar_compile_support::compile(text.as_bytes(), ScalarCompilationBudget::default())
                .expect_err("Unsupported scalar value");
        assert_eq!(error.kind, ScalarCompileErrorKind::UnsupportedPrimary);
        assert!(error.work_steps > 0);
        let diagnostic = format!("{error:?} {error}");
        assert!(!diagnostic.contains("PrivateMain"));
        assert!(!diagnostic.contains("private_"));
        assert!(!diagnostic.contains("private-value"));
    }
    for expression in ["()", "(1,2)", "(1,2,3)"] {
        let text = format!("main(){{return {expression};}}");
        assert_eq!(
            scalar_compile_support::compile(text.as_bytes(), ScalarCompilationBudget::default())
                .expect_err("Scalar group arity")
                .kind,
            ScalarCompileErrorKind::UnsupportedGroupArity,
        );
    }
}
