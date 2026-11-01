// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source lowering selects every scalar operator and preserves numeric domains.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid synthetic scalar modules."
)]

use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::literal::LegacyLiteralError;
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn all_numeric_operators_and_integer_boundaries_lower_to_actual_results() -> Result<(), String> {
    for (expression, expected) in [
        ("8|3", 11),
        ("8^3", 11),
        ("8&3", 0),
        ("2==2", 1),
        ("2!=3", 1),
        ("2<3", 1),
        ("3>2", 1),
        ("2<=2", 1),
        ("2>=2", 1),
        ("3<<2", 12),
        ("12>>2", 3),
        ("3+2", 5),
        ("3-2", 1),
        ("3*2", 6),
        ("7%3", 1),
        ("!0", 1),
        ("!1.5", 0),
        ("~3", -4),
        ("2147483647+1", i32::MIN),
        ("-2147483648-1", i32::MAX),
        ("-2147483648", i32::MIN),
        ("-2147483648>>31", -1),
        ("1<<-1", i32::MIN),
        ("-7%3", -1),
        ("16777217==16777216.0", 1),
        ("false || true && false", 0),
    ] {
        assert_eq!(
            evaluate(expression),
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
    for (expression, bits) in [
        ("7/2", 3.5_f32.to_bits()),
        ("1+2.5", 3.5_f32.to_bits()),
        ("-0.0", 0x8000_0000),
        ("-0.0/2", 0x8000_0000),
        ("16777217+0.0", 16_777_216_f32.to_bits()),
    ] {
        let KernelValue::Number(LegacyNumber::Float(value)) = evaluate(expression) else {
            return Err(format!(
                "Expected float for synthetic expression {expression}"
            ));
        };
        assert_eq!(value.to_bits(), bits);
    }
    Ok(())
}

#[test]
fn signed_literal_and_numeric_operation_failures_keep_source_and_work_without_values()
-> Result<(), String> {
    for (expression, expected, token) in [
        (
            "2147483648",
            FunctionCompileErrorKind::Literal(LegacyLiteralError::NumberOutOfRange),
            "2147483648",
        ),
        (
            "-2147483649",
            FunctionCompileErrorKind::Literal(LegacyLiteralError::NumberOutOfRange),
            "2147483649",
        ),
    ] {
        let text = format!("PrivateMain(){{return {expression};}}");
        let error = function_compile_support::compile_scalar(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Checked literal domain");
        assert_eq!(error.kind, expected);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source.span(error.span.clone()).expect("Literal span"),
            token.as_bytes()
        );
        assert!(error.work_steps > 0);
        assert!(!format!("{error:?} {error}").contains("PrivateMain"));
    }
    for (expression, expected, token) in [
        (
            "-2147483648%-1",
            KernelErrorKind::Numeric(LegacyNumberError::RemainderOverflow),
            "%",
        ),
        (
            "1.0&1",
            KernelErrorKind::Numeric(LegacyNumberError::ExpectedInteger),
            "&",
        ),
        ("undefined+1", KernelErrorKind::ExpectedNumber, "+"),
    ] {
        let text = format!("PrivateMain(){{return {expression};}}");
        let error = function_compile_support::compile_scalar(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Pure validation rejects numeric errors");
        let FunctionCompileErrorKind::Kernel(kernel) = error.kind else {
            return Err(format!("Expected kernel error, got {:?}", error.kind));
        };
        assert_eq!(kernel.kind, expected);
        assert!(kernel.work_steps > 0);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source.span(error.span).expect("Operator span"),
            token.as_bytes()
        );
    }
    Ok(())
}

fn evaluate(expression: &str) -> KernelValue {
    let text = format!("main(){{return {expression};}}");
    function_compile_support::compile_scalar(text.as_bytes(), FunctionCompilationBudget::default())
        .expect("Scalar compilation")
        .program()
        .execute(KernelExecutionBudget::default())
        .expect("Scalar execution")
        .value
}
