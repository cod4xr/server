// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-to-owned-code execution through the explicit pure scalar function profile.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid synthetic modules and expected scalar programs."
)]

use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::scalar_compile_support;

#[test]
fn source_functions_execute_empty_void_and_numeric_returns_after_source_and_trees_drop() {
    for (text, expected) in [
        ("PrivateMain(){}", KernelValue::Undefined),
        ("PrivateMain(){return;}", KernelValue::Undefined),
        (
            "PrivateMain(){return 1+2*3;}",
            KernelValue::Number(LegacyNumber::Integer(7)),
        ),
        (
            "PrivateMain(){return (1+2)*3;}",
            KernelValue::Number(LegacyNumber::Integer(9)),
        ),
    ] {
        let mut bytes = text.as_bytes().to_vec();
        let compiled = scalar_compile_support::compile(&bytes, ScalarCompilationBudget::default())
            .expect("Source compilation");
        bytes.fill(0);
        let outcome = compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect("Owned source program");
        assert_eq!(outcome.value, expected);
        assert_eq!(
            compiled.usage().instructions,
            compiled.program().usage().instructions
        );
        assert!(!format!("{compiled:?}").contains("PrivateMain"));
    }
}

#[test]
fn source_short_circuit_branches_skip_numeric_errors_and_normalize_bool_results() {
    for (expression, expected) in [
        ("false && (1/0)", 0),
        ("true || (1/0)", 1),
        ("0.0 || 1.5", 1),
        ("1.5 && 0.0", 0),
        ("(false || (true && false)) || true", 1),
    ] {
        let text = format!("main(){{return {expression};}}");
        let compiled =
            scalar_compile_support::compile(text.as_bytes(), ScalarCompilationBudget::default())
                .expect("Lazy source compilation");
        let outcome = compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect("Lazy execution");
        assert_eq!(
            outcome.value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
        assert_eq!(outcome.steps, compiled.usage().validation_steps);
    }
    let text = b"main(){return true && (1/0);}";
    let error = scalar_compile_support::compile(text, ScalarCompilationBudget::default())
        .expect_err("Taken error during pure validation");
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(source.span(error.span.clone()).expect("Error span"), b"/");
    assert!(
        matches!(error.kind, ScalarCompileErrorKind::Kernel(error) if error.kind == KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero))
    );
}

#[test]
fn instruction_maps_keep_original_numeric_operator_and_return_byte_spans() {
    let text = b"PrivateMain(){\n return 12 / 3 + 1;\n}";
    let compiled = scalar_compile_support::compile(text, ScalarCompilationBudget::default())
        .expect("Mapped function");
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    for (index, expected) in [
        b"12".as_slice(),
        b"3",
        b"/",
        b"1",
        b"+",
        b"return 12 / 3 + 1",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            source
                .span(
                    compiled
                        .instruction_span(index as u64)
                        .expect("Instruction span")
                )
                .expect("Original source span"),
            expected
        );
    }
    assert!(compiled.instruction_span(6).is_none());
    assert!(compiled.instruction_span(u64::MAX).is_none());
}
