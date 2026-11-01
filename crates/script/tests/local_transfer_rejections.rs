// SPDX-License-Identifier: AGPL-3.0-or-later
//! Invalid transfer contexts and source reachability remain explicit located errors.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid syntax with restricted semantic failures."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn illegal_contexts_restored_outer_context_and_unreachable_source_fail() {
    for (text, kind, token) in [
        (
            "PrivateMain(){break;}",
            ScalarCompileErrorKind::IllegalBreak,
            "break",
        ),
        (
            "PrivateMain(){continue;}",
            ScalarCompileErrorKind::IllegalContinue,
            "continue",
        ),
        (
            "PrivateMain(arg){while(arg){break;}break;}",
            ScalarCompileErrorKind::IllegalBreak,
            "break",
        ),
        (
            "PrivateMain(arg){while(arg){continue;}continue;}",
            ScalarCompileErrorKind::IllegalContinue,
            "continue",
        ),
        (
            "PrivateMain(arg){while(arg){break;PrivateLocal=1;}}",
            ScalarCompileErrorKind::UnreachableStatement,
            "PrivateLocal=1;",
        ),
        (
            "PrivateMain(arg){while(arg){if(arg)break;else continue;PrivateLocal=1;}}",
            ScalarCompileErrorKind::UnreachableStatement,
            "PrivateLocal=1;",
        ),
        (
            "PrivateMain(arg){while(arg){PrivateLocal=1;break;}return PrivateLocal;}",
            ScalarCompileErrorKind::UninitializedLocal,
            "PrivateLocal",
        ),
        (
            "PrivateMain(arg){while(arg){if(arg){PrivateLocal=1;continue;}else{break;}}return PrivateLocal;}",
            ScalarCompileErrorKind::UninitializedLocal,
            "PrivateLocal",
        ),
    ] {
        let error = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect_err("Restricted transfer failure");
        assert_eq!(error.kind, kind);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source.span(error.span.clone()).expect("Original location"),
            token.as_bytes()
        );
        assert!(!format!("{error:?} {error}").contains("Private"));
        assert!(error.work_steps > 0);
    }
}
