// SPDX-License-Identifier: AGPL-3.0-or-later
//! Restricted local binding rejects invalid initialization and unsupported source explicitly.
#![allow(
    clippy::expect_used,
    reason = "Tests supply syntactically valid restricted-profile failures."
)]

use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn first_self_read_late_missing_and_short_circuited_uninitialized_names_fail_at_source() {
    for (text, token) in [
        (
            "main(){PrivateLocal=PrivateLocal+1;return PrivateLocal;}",
            "PrivateLocal",
        ),
        (
            "main(){x=PrivateLater;PrivateLater=1;return x;}",
            "PrivateLater",
        ),
        ("main(){return PrivateMissing;}", "PrivateMissing"),
        ("main(){return true || PrivateMissing;}", "PrivateMissing"),
    ] {
        let error = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Initialization is checked at compile time");
        assert_eq!(error.kind, FunctionCompileErrorKind::UninitializedLocal);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source.span(error.span.clone()).expect("Name span"),
            token.as_bytes()
        );
        assert!(error.work_steps > 0);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn duplicate_parameters_nonfinal_returns_and_non_name_targets_fail_under_this_profile() {
    for (text, expected) in [
        (
            "main(PrivateArg,privatearg){return 1;}",
            FunctionCompileErrorKind::DuplicateParameter,
        ),
        (
            "main(){return 1;x=2;}",
            FunctionCompileErrorKind::ReturnNotLast,
        ),
        (
            "main(){return 1;;}",
            FunctionCompileErrorKind::ReturnNotLast,
        ),
        (
            "main(){self.PrivateField=1;}",
            FunctionCompileErrorKind::UnsupportedLocalTarget,
        ),
        (
            "main(){PrivateArray[0]=1;}",
            FunctionCompileErrorKind::UnsupportedLocalTarget,
        ),
    ] {
        let error = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Explicit restricted policy");
        assert_eq!(error.kind, expected);
        assert!(error.work_steps > 0);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn general_control_flow_richer_values_and_calls_remain_explicitly_unsupported() {
    for text in ["main(){wait 1;}", "main(){PrivateCall();}"] {
        assert_eq!(
            function_compile_support::compile_primitive(
                text.as_bytes(),
                FunctionCompilationBudget::default()
            )
            .expect_err("Unsupported body")
            .kind,
            FunctionCompileErrorKind::UnsupportedBody
        );
    }
    for text in [
        "main(){return self;}",
        "main(){x=\"PrivateString\";return x;}",
        "main(){return true || PrivateCall();}",
        "main(){return (1,2);}",
    ] {
        let error = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Unsupported value");
        assert!(matches!(
            error.kind,
            FunctionCompileErrorKind::UnsupportedPrimary
                | FunctionCompileErrorKind::UnsupportedGroupArity
        ));
        assert!(!format!("{error:?}").contains("Private"));
    }
}
