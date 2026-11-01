// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source equality retains initialized Undefined, legacy errors, maps and lazy flow.
#![allow(
    clippy::expect_used,
    reason = "Tests provide valid primitive source and selected failures."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn both_source_profiles_compile_equal_undefined_and_preserve_lazy_comparisons() {
    for (expression, expected) in [
        ("undefined==undefined", 1),
        ("undefined!=undefined", 0),
        ("!(undefined!=undefined)", 1),
        ("false&&(undefined==1)", 0),
        ("true||(1!=undefined)", 1),
        ("(undefined==undefined)&&(undefined!=undefined)", 0),
    ] {
        let text = format!("PrivateMain(){{return {expression};}}");
        for compile in [
            function_compile_support::compile_scalar,
            function_compile_support::compile_primitive,
        ] {
            let function = compile(text.as_bytes(), FunctionCompilationBudget::default())
                .expect("Compile source comparison");
            assert_eq!(
                function
                    .program()
                    .execute(KernelExecutionBudget::default())
                    .expect("Defined comparison")
                    .value,
                KernelValue::Number(LegacyNumber::Integer(expected))
            );
            assert!(!format!("{function:?}").contains("PrivateMain"));
        }
    }
}

#[test]
fn missing_arguments_and_explicit_stored_undefined_compare_without_initializing_other_slots() {
    let function = function_compile_support::compile_primitive(
        b"main(a,b){u=undefined;if(a==u){return b==undefined;}else{return u!=undefined;}}",
        FunctionCompilationBudget::default(),
    )
    .expect("Initialized Undefined is readable");
    let arguments = [KernelValue::Undefined];
    assert_eq!(
        function
            .program()
            .execute_legacy_script_arguments(&arguments, KernelExecutionBudget::default())
            .expect("Missing b receives Undefined")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    assert_eq!(arguments, [KernelValue::Undefined]);
    let error = function_compile_support::compile_primitive(
        b"main(){return privateUninitialized==undefined;}",
        FunctionCompilationBudget::default(),
    )
    .expect_err("No implicit local creation");
    assert_eq!(error.kind, FunctionCompileErrorKind::UninitializedLocal);
}

#[test]
fn undefined_pair_errors_retain_operator_locations_and_pure_validation() {
    for expression in ["a==1", "1!=a", "a<(1,2,3)", "(1,2,3)>=a", "a<a"] {
        let text = format!("PrivateMain(a){{return {expression};}}");
        let function = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Runtime comparison source");
        let error = function
            .program()
            .execute_with_arguments(&[KernelValue::Undefined], KernelExecutionBudget::default())
            .expect_err("Incompatible undefined pair");
        assert_eq!(error.kind, KernelErrorKind::UnmatchingTypes);
        let span = function
            .instruction_span(error.instruction.expect("Comparison PC"))
            .expect("Operator span");
        let start = usize::try_from(span.start).expect("Offset");
        let end = usize::try_from(span.end).expect("Offset");
        assert!(matches!(
            &text.as_bytes()[start..end],
            b"==" | b"!=" | b"<" | b">="
        ));
        assert!(!format!("{error:?} {error}").contains("PrivateMain"));
    }
    let error = function_compile_support::compile_scalar(
        b"main(){return undefined==1;}",
        FunctionCompilationBudget::default(),
    )
    .expect_err("Pure validation fails before publication");
    assert!(matches!(
        error.kind,
        FunctionCompileErrorKind::Kernel(error) if error.kind == KernelErrorKind::UnmatchingTypes
    ));
}
