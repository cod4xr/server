// SPDX-License-Identifier: AGPL-3.0-or-later
//! Standalone plain-name postfix statements retain integer-only behavior and source locations.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid finite postfix statement source."
)]

use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn postfix_statements_update_integer_locals_and_private_parameters_with_wrapping() {
    for (text, expected) in [
        ("main(){x=1;x++;return x;}", 2),
        ("main(){x=1;x--;return x;}", 0),
        ("main(){x=2147483647;x++;return x;}", i32::MIN),
        ("main(){x=-2147483648;x--;return x;}", i32::MAX),
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Postfix source compilation");
        assert_eq!(
            compiled
                .program()
                .execute(KernelExecutionBudget::default())
                .expect("Postfix source invocation")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
        assert_eq!(compiled.usage().local_bindings, 1);
    }
    let compiled = function_compile_support::compile_primitive(
        b"main(arg){arg++;arg--;return arg;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Postfix parameter source");
    let arguments = [KernelValue::Number(LegacyNumber::Integer(i32::MAX))];
    assert_eq!(
        compiled
            .program()
            .execute_with_arguments(&arguments, KernelExecutionBudget::default())
            .expect("Private parameter updates")
            .value,
        arguments[0]
    );
    assert_eq!(
        arguments[0],
        KernelValue::Number(LegacyNumber::Integer(i32::MAX))
    );
}

#[test]
fn postfix_requires_initialized_plain_targets_and_rejects_floats_at_original_update_tokens() {
    for (text, expected) in [
        (
            "main(){PrivateMissing++;}",
            FunctionCompileErrorKind::UninitializedLocal,
        ),
        (
            "main(){self.PrivateField--;}",
            FunctionCompileErrorKind::UnsupportedLocalTarget,
        ),
        (
            "main(){PrivateArray[0]++;}",
            FunctionCompileErrorKind::UnsupportedLocalTarget,
        ),
    ] {
        let error = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect_err("Postfix target admission");
        assert_eq!(error.kind, expected);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
    for (text, expected, token) in [
        (
            "main(){x=1.0;x++;return x;}",
            KernelErrorKind::Numeric(LegacyNumberError::ExpectedInteger),
            "++",
        ),
        (
            "main(){x=-0.0;x--;return x;}",
            KernelErrorKind::Numeric(LegacyNumberError::ExpectedInteger),
            "--",
        ),
        (
            "main(){x=undefined;x++;return x;}",
            KernelErrorKind::ExpectedNumber,
            "++",
        ),
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Located update function");
        let error = compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect_err("Integer-only runtime update");
        assert_eq!(error.kind, expected);
        let span = compiled
            .instruction_span(error.instruction.expect("Located instruction"))
            .expect("Original update span");
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(source.span(span).expect("Update token"), token.as_bytes());
        assert!(error.work_steps > 0);
    }
}

#[test]
fn postfix_source_uses_existing_name_task_code_and_binding_admission() {
    let text = b"main(arg){x=arg;x++;x--;return x;}";
    let usage =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Baseline")
            .usage();
    assert_eq!(usage.identifier_bytes, 13);
    assert_eq!(usage.local_bindings, 2);
    let exact = FunctionCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_identifier_bytes: usage.identifier_bytes,
        max_local_bindings: usage.local_bindings,
        ..FunctionCompilationBudget::default()
    };
    assert_eq!(
        function_compile_support::compile_primitive(text, exact)
            .expect("Exact postfix admission")
            .usage(),
        usage
    );
    let error = function_compile_support::compile_primitive(
        text,
        FunctionCompilationBudget {
            max_identifier_bytes: 12,
            ..exact
        },
    )
    .expect_err("All postfix visits stay admitted");
    assert!(matches!(
        error.kind,
        FunctionCompileErrorKind::BudgetExceeded { .. }
    ));
    assert!(error.work_steps > 0);
}
