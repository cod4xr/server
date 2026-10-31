// SPDX-License-Identifier: AGPL-3.0-or-later
//! Restricted local binding rejects invalid initialization and unsupported source explicitly.
#![allow(
    clippy::expect_used,
    reason = "Tests supply syntactically valid restricted-profile failures."
)]

use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::scalar_compile_support;

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
        let error = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect_err("Initialization is checked at compile time");
        assert_eq!(error.kind, ScalarCompileErrorKind::UninitializedLocal);
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
            ScalarCompileErrorKind::DuplicateParameter,
        ),
        (
            "main(){return 1;x=2;}",
            ScalarCompileErrorKind::ReturnNotLast,
        ),
        ("main(){return 1;;}", ScalarCompileErrorKind::ReturnNotLast),
        (
            "main(){self.PrivateField=1;}",
            ScalarCompileErrorKind::UnsupportedLocalTarget,
        ),
        (
            "main(){PrivateArray[0]=1;}",
            ScalarCompileErrorKind::UnsupportedLocalTarget,
        ),
    ] {
        let error = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect_err("Explicit restricted policy");
        assert_eq!(error.kind, expected);
        assert!(error.work_steps > 0);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn general_control_flow_updates_values_and_calls_remain_explicitly_unsupported() {
    for text in [
        "main(){if(true)return 1;}",
        "main(){while(true){}}",
        "main(){{x=1;}}",
        "main(){x=1;x++;}",
        "main(){wait 1;}",
        "main(){PrivateCall();}",
    ] {
        assert_eq!(
            scalar_compile_support::compile_local(
                text.as_bytes(),
                ScalarCompilationBudget::default()
            )
            .expect_err("Unsupported body")
            .kind,
            ScalarCompileErrorKind::UnsupportedBody
        );
    }
    for text in [
        "main(){return self;}",
        "main(){x=\"PrivateString\";return x;}",
        "main(){return true || PrivateCall();}",
        "main(){return (1,2,3);}",
    ] {
        let error = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect_err("Unsupported value");
        assert!(matches!(
            error.kind,
            ScalarCompileErrorKind::UnsupportedPrimary
                | ScalarCompileErrorKind::UnsupportedGroupArity
        ));
        assert!(!format!("{error:?}").contains("Private"));
    }
}
