// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source blocks and branches execute owned verified code with original spans.
#![allow(
    clippy::expect_used,
    reason = "Tests supply finite valid synthetic branch functions."
)]

use cod4x_script::compilation::FunctionCompilationBudget;
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn branches_blocks_else_if_and_returning_paths_select_real_results() {
    for (text, results) in [
        (
            "main(arg){if(arg){x=11;}else{x=22;}return x;}",
            [22, 11, 11],
        ),
        ("main(arg){{x=3;{x+=2;}}if(arg)x++;return x;}", [5, 6, 6]),
        (
            "main(arg){if(arg==0)return 7;else if(arg==1)return 8;else return 9;}",
            [7, 8, 9],
        ),
        ("main(arg){if(arg)return 4;x=6;return x;}", [6, 4, 4]),
        (
            "main(arg){if(arg){x=5;}else{return 8;}return x;}",
            [8, 5, 5],
        ),
        (
            "main(arg){if(arg){return 8;}else{x=5;}return x;}",
            [5, 8, 8],
        ),
        (
            "main(arg){if(arg){if(arg==1)x=2;else x=4;}else x=3;return x;}",
            [3, 2, 4],
        ),
    ] {
        let mut source = text.as_bytes().to_vec();
        let compiled = function_compile_support::compile_primitive(
            &source,
            FunctionCompilationBudget::default(),
        )
        .expect("Branch compilation");
        source.fill(0);
        assert_eq!(compiled.usage().validation_steps, None);
        for (argument, expected) in (0..3).zip(results) {
            let args = [KernelValue::Number(LegacyNumber::Integer(argument))];
            let result = compiled
                .program()
                .execute_with_arguments(&args, KernelExecutionBudget::default())
                .expect("Chosen path");
            assert_eq!(
                result.value,
                KernelValue::Number(LegacyNumber::Integer(expected)),
                "{text}"
            );
            assert_eq!(
                args[0],
                KernelValue::Number(LegacyNumber::Integer(argument))
            );
        }
    }
}

#[test]
fn empty_branches_blocks_and_bare_returns_preserve_undefined() {
    for text in [
        "main(arg){if(arg){}else{}}",
        "main(arg){if(arg){return;}else{{return;}}}",
        "main(arg){if(arg){}else if(arg){};}",
        "main(arg){{}if(arg)return;}",
        "main(arg){if(arg){x=undefined;}else{x=undefined;}return x;}",
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Empty branch source");
        for arg in [0, 1] {
            assert_eq!(
                compiled
                    .program()
                    .execute_with_arguments(
                        &[KernelValue::Number(LegacyNumber::Integer(arg))],
                        KernelExecutionBudget::default()
                    )
                    .expect("Empty branch invocation")
                    .value,
                KernelValue::Undefined
            );
        }
    }
}

#[test]
fn condition_type_errors_and_taken_operator_errors_map_to_source() {
    let text = b"PrivateMain(arg){if(arg){return 1/0;}else{return 7;}}";
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Runtime-dependent source");
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    for (arg, kind, token) in [
        (
            KernelValue::Undefined,
            KernelErrorKind::ExpectedNumber,
            b"if".as_slice(),
        ),
        (
            KernelValue::Number(LegacyNumber::Integer(1)),
            KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero),
            b"/".as_slice(),
        ),
    ] {
        let error = compiled
            .program()
            .execute_with_arguments(&[arg], KernelExecutionBudget::default())
            .expect_err("Mapped runtime failure");
        assert_eq!(error.kind, kind);
        let span = compiled
            .instruction_span(error.instruction.expect("Located error"))
            .expect("Operation span");
        assert_eq!(source.span(span).expect("Original token"), token);
        assert!(!format!("{error:?} {compiled:?}").contains("Private"));
    }
    assert_eq!(
        compiled
            .program()
            .execute_with_arguments(
                &[KernelValue::Number(LegacyNumber::Integer(0))],
                KernelExecutionBudget::default()
            )
            .expect("Untaken failure")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
}
