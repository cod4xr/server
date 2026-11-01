// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primitive constructor order is explicit; pure/unsupported source stays restricted.
#![allow(
    clippy::expect_used,
    reason = "Tests supply syntactically valid source and selected failures."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError, LegacyVectorError};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn vector_component_failures_execute_right_to_left_with_original_operation_maps() {
    let text = b"PrivateMain(a){return(1/a,2/a,3/a);}";
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Dynamic component source");
    let error = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(0))],
            KernelExecutionBudget::default(),
        )
        .expect_err("Rightmost component fails first");
    assert_eq!(
        error.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    let offset = u64::try_from(
        text.iter()
            .rposition(|byte| *byte == b'/')
            .expect("Last division"),
    )
    .expect("Source offset");
    assert_eq!(
        compiled.instruction_span(error.instruction.expect("PC")),
        Some(offset..offset + 1)
    );
    for (text, kind, token) in [
        (
            "PrivateMain(){return(undefined,2,3);}",
            KernelErrorKind::ExpectedNumber,
            "(undefined,2,3)",
        ),
        (
            "PrivateMain(){return((1,2,3),2,3);}",
            KernelErrorKind::ExpectedNumber,
            "((1,2,3),2,3)",
        ),
        (
            "PrivateMain(){return(1,2,3)+1;}",
            KernelErrorKind::Vector(LegacyVectorError::UnsupportedMixedAddition),
            "+",
        ),
        (
            "PrivateMain(){return(1,2,3)/(1,0,1);}",
            KernelErrorKind::Vector(LegacyVectorError::DivisionByZero),
            "/",
        ),
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Invocation-only validation");
        let error = compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect_err("Located vector failure");
        assert_eq!(error.kind, kind);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source
                .span(
                    compiled
                        .instruction_span(error.instruction.expect("PC"))
                        .expect("Map")
                )
                .expect("Operation span"),
            token.as_bytes()
        );
        assert!(!format!("{compiled:?} {error:?}").contains("Private"));
    }
}

#[test]
fn group_shape_profile_initialization_and_skipped_source_checks_remain_explicit() {
    for text in [
        "main(){return();}",
        "main(){return(1,2);}",
        "main(){return(1,2,3,4);}",
        "main(){return true || (1,2);}",
    ] {
        assert_eq!(
            function_compile_support::compile_primitive(
                text.as_bytes(),
                FunctionCompilationBudget::default()
            )
            .expect_err("Unsupported shape")
            .kind,
            FunctionCompileErrorKind::UnsupportedGroupArity
        );
    }
    assert_eq!(
        function_compile_support::compile_scalar(
            b"main(){return(1,2,3);}",
            FunctionCompilationBudget::default()
        )
        .expect_err("Unchanged pure scalar profile")
        .kind,
        FunctionCompileErrorKind::UnsupportedGroupArity
    );
    let text = b"PrivateMain(){return(PrivateX,PrivateY,PrivateZ);}";
    let error =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect_err("Names are checked in explicit component order");
    assert_eq!(error.kind, FunctionCompileErrorKind::UninitializedLocal);
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(
        source.span(error.span).expect("First missing component"),
        b"PrivateZ"
    );
    assert_eq!(
        function_compile_support::compile_primitive(
            b"main(){return true || (1,2,PrivateCall());}",
            FunctionCompilationBudget::default()
        )
        .expect_err("Skipped unsupported component source")
        .kind,
        FunctionCompileErrorKind::UnsupportedPrimary
    );
    assert_eq!(
        function_compile_support::compile_primitive(
            b"main(){x=(1,2,x);return x;}",
            FunctionCompilationBudget::default()
        )
        .expect_err("New targets initialize after RHS")
        .kind,
        FunctionCompileErrorKind::UninitializedLocal
    );
}
