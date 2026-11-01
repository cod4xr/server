// SPDX-License-Identifier: AGPL-3.0-or-later
//! For updates merge continuing paths and conservative exits restore incoming facts.
#![allow(
    clippy::expect_used,
    reason = "Tests use syntactically valid scalar for profiles."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn update_reads_accept_only_initialization_on_every_continuing_path() {
    for text in [
        "main(n){for(i=0;i<n;x++){if(i%2==0){x=10;i++;continue;}x=20;i++;}return i;}",
        "main(n){for(i=0;i<n;x++){if(i%3==0){x=1;i++;continue;}if(i%3==1){x=2;i++;continue;}x=3;i++;}return i;}",
        "main(n){for(i=0;i<n;x++){if(i==4)break;x=1;i++;}return i;}",
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Update facts from reaching paths");
        for arg in 0..4 {
            assert_eq!(
                compiled
                    .program()
                    .execute_with_arguments(
                        &[KernelValue::Number(LegacyNumber::Integer(arg))],
                        KernelExecutionBudget::default()
                    )
                    .expect("Initialized update invocation")
                    .value,
                KernelValue::Number(LegacyNumber::Integer(arg))
            );
        }
    }
}

#[test]
fn missing_continue_exit_and_unreachable_update_reads_remain_located_failures() {
    for text in [
        "PrivateMain(n){for(i=0;i<n;PrivateLocal++){if(i==0){i++;continue;}PrivateLocal=1;i++;}}",
        "PrivateMain(n){for(i=0;i<n;){PrivateLocal=1;break;}return PrivateLocal;}",
        "PrivateMain(){for(i=0;i<3;PrivateLocal++){PrivateLocal=1;break;}}",
        "PrivateMain(){for(;false;PrivateLocal++){}}",
    ] {
        let error = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Conservative initialization failure");
        assert_eq!(error.kind, FunctionCompileErrorKind::UninitializedLocal);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source.span(error.span.clone()).expect("Read location"),
            b"PrivateLocal"
        );
        assert!(error.work_steps > 0);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
    for text in [
        "main(){for(PrivateCall();;){}}",
        "main(){for(;;PrivateCall()){}}",
    ] {
        assert_eq!(
            function_compile_support::compile_primitive(
                text.as_bytes(),
                FunctionCompilationBudget::default()
            )
            .expect_err("Restricted clause policy")
            .kind,
            FunctionCompileErrorKind::UnsupportedForClause
        );
    }
}
