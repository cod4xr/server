// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic labels exercise decoded duplicates and deliberately separate compiler profiles."
)]
use crate::{function_compile_support, module_compile_support, source_scope_support::compile};
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompileErrorKind, ModuleCompilationBudget,
};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelExecutionBudget, KernelValue,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

#[test]
fn decoded_duplicate_labels_report_the_second_source_case_without_exposing_label_bytes() {
    for source in [
        br#"main(k){switch(k){case "Private":break;case "other":break;case "Private":break;}}"#
            .as_slice(),
        br#"main(k){switch(k){case "a\"b":break;case 1:break;case "a\"b":break;}}"#,
        br#"main(k){switch(k){case "":break;case 0:break;case "":break;}}"#,
    ] {
        let error =
            module_compile_support::compile(source, Disabled, ModuleCompilationBudget::default())
                .expect_err("Decoded identity rejects duplicate rows");
        assert_eq!(error.cause.kind, FunctionCompileErrorKind::DuplicateCase);
        let second = source
            .windows(5)
            .rposition(|bytes| bytes == b"case ")
            .expect("Second case");
        assert_eq!(error.cause.span.start, second as u64);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn localized_grouped_boolean_float_and_expression_cases_remain_rejected() {
    for label in ["&\"Private\"", "(\"Private\")", "true", "1.0", "undefined"] {
        let source = format!("main(k){{switch(k){{case {label}:break;}}}}");
        let error = module_compile_support::compile(
            source.as_bytes(),
            Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect_err("Only integer and plain-string literal productions");
        assert_eq!(error.cause.kind, FunctionCompileErrorKind::UnsupportedCase);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
    // Binary case expressions fail earlier in the pinned grammar, before lowering.
    let source = SourceText::index(
        b"main(k){switch(k){case 1+2:break;}}",
        SourceBudget::default(),
    )
    .expect("Source");
    LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect_err("Case expressions are not a grammar production");
}

#[test]
fn standalone_integer_profile_keeps_its_source_gate_and_strict_key_type() {
    function_compile_support::compile_primitive(
        br#"main(k){switch(k){case "key":return 1;}return 0;}"#,
        FunctionCompilationBudget::default(),
    )
    .expect_err("Standalone string labels remain outside the bound profile");
    let function = function_compile_support::compile_primitive(
        b"main(k){switch(k){case 1:return 1;}return 0;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Standalone integer switch");
    // Bound modules admit two domains while standalone source keeps RequireInteger.
    let module = compile(b"main(k){switch(k){case 1:return 1;}return 0;}");
    assert_eq!(
        module
            .program()
            .execute(0, &[KernelValue::Undefined], KernelCallBudget::default())
            .expect_err("Bound key validation")
            .cause
            .kind,
        KernelErrorKind::ExpectedSwitchKey
    );
    for key in [
        KernelValue::Undefined,
        KernelValue::Number(LegacyNumber::Float(1.0)),
    ] {
        assert_eq!(
            function
                .program()
                .execute_with_arguments(&[key], KernelExecutionBudget::default())
                .expect_err("Standalone strict integer key")
                .kind,
            KernelErrorKind::ExpectedInteger
        );
        assert_eq!(
            module
                .program()
                .execute(0, &[key], KernelCallBudget::default())
                .expect_err("No coercion")
                .cause
                .kind,
            KernelErrorKind::ExpectedSwitchKey
        );
    }
}
