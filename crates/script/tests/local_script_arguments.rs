// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-compiled scalar parameters use explicit legacy defaults with mapped runtime errors.
#![allow(
    clippy::expect_used,
    reason = "Tests use valid finite source and ordered scalar arguments."
)]

use cod4x_script::compilation::FunctionCompilationBudget;
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn source_parameter_order_missing_undefined_and_excess_errors_are_explicit() {
    let compiled = function_compile_support::compile_primitive(
        b"main(first,second){return second;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Source function");
    let arguments = [
        KernelValue::Number(LegacyNumber::Integer(7)),
        KernelValue::Number(LegacyNumber::Integer(11)),
    ];
    for count in 0..=2 {
        let outcome = compiled
            .program()
            .execute_legacy_script_arguments(&arguments[..count], KernelExecutionBudget::default())
            .expect("Explicit legacy invocation");
        assert_eq!(
            outcome.value,
            if count == 2 {
                arguments[1]
            } else {
                KernelValue::Undefined
            }
        );
    }
    let error = compiled
        .program()
        .execute_legacy_script_arguments(
            &[KernelValue::Undefined; 3],
            KernelExecutionBudget::default(),
        )
        .expect_err("Excess source arguments");
    assert_eq!(
        error.kind,
        KernelErrorKind::ArgumentCount {
            expected: 2,
            actual: 3
        }
    );
    assert_eq!(error.instruction, None);
    assert_eq!(compiled.usage().validation_steps, None);
}

#[test]
fn missing_parameter_is_initialized_but_numeric_use_still_fails_at_original_operator() {
    let text = b"main(arg){return arg+1;}";
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Source parameter binding");
    let error = compiled
        .program()
        .execute_legacy_script_arguments(&[], KernelExecutionBudget::default())
        .expect_err("Undefined numeric operand");
    assert_eq!(error.kind, KernelErrorKind::ExpectedNumber);
    let span = compiled
        .instruction_span(error.instruction.expect("Located operation"))
        .expect("Original span");
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(source.span(span).expect("Operator"), b"+");
    assert!(error.work_steps > 0);
    assert_eq!(error.admitted_local_values, 1);
}
