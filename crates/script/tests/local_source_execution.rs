// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source names/assignments lower into owned executable scalar parameter/local code.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid synthetic local-profile functions."
)]

use cod4x_script::compilation::{FunctionCompilationBudget, FunctionSourceProfile};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::function_compile_support;

#[test]
fn parameters_assignments_reassignment_case_and_source_independent_ownership_execute() {
    let mut source = b"PrivateMain(PrivateArg,other){;x=PrivateArg+other;X=x*2;return x;}".to_vec();
    let compiled =
        function_compile_support::compile_primitive(&source, FunctionCompilationBudget::default())
            .expect("Local source compilation");
    source.fill(0);
    assert_eq!(
        compiled.profile(),
        FunctionSourceProfile::PrimitiveStatements
    );
    assert_eq!(compiled.usage().validation_steps, None);
    assert_eq!(compiled.usage().local_bindings, 3);
    assert_eq!(compiled.program().layout().parameters(), 2);
    assert_eq!(compiled.program().layout().local_values(), 3);
    let arguments = [
        KernelValue::Number(LegacyNumber::Integer(3)),
        KernelValue::Number(LegacyNumber::Integer(4)),
    ];
    let outcome = compiled
        .program()
        .execute_with_arguments(&arguments, KernelExecutionBudget::default())
        .expect("Source local invocation");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(14))
    );
    assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(3)));
    let mutate_parameter = function_compile_support::compile_primitive(
        b"main(arg){arg=arg+1;return arg;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Parameter reassignment");
    assert_eq!(
        mutate_parameter
            .program()
            .execute_with_arguments(&arguments[..1], KernelExecutionBudget::default())
            .expect("Private parameter mutation")
            .value,
        KernelValue::Number(LegacyNumber::Integer(4))
    );
    assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(3)));
    assert!(!format!("{compiled:?}").contains("Private"));
    let pure = function_compile_support::compile_scalar(
        b"main(){return 1;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Original profile");
    assert_eq!(pure.profile(), FunctionSourceProfile::PureScalarReturn);
    assert!(pure.usage().validation_steps.is_some());
}

#[test]
fn empty_bare_return_fallthrough_and_initialized_undefined_have_real_results() {
    for text in [
        b"main(){}".as_slice(),
        b"main(){return;}",
        b"main(){x=1;;}",
        b"main(){x=undefined;return x;}",
    ] {
        let compiled =
            function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
                .expect("Local empty/undefined source");
        assert_eq!(
            compiled
                .program()
                .execute(KernelExecutionBudget::default())
                .expect("Source invocation")
                .value,
            KernelValue::Undefined
        );
        assert_eq!(compiled.usage().validation_steps, None);
    }
}

#[test]
fn lazy_parameter_expressions_execute_chosen_path_with_mapped_numeric_errors() {
    let source = b"PrivateMain(arg){x=arg;return x || (1/0);}";
    let compiled = function_compile_support::compile_primitive(
        source,
        FunctionCompilationBudget {
            validation: KernelExecutionBudget {
                max_steps: 0,
                ..KernelExecutionBudget::default()
            },
            ..FunctionCompilationBudget::default()
        },
    )
    .expect("No fabricated validation arguments");
    let truth = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(7))],
            KernelExecutionBudget::default(),
        )
        .expect("Lazy true argument");
    assert_eq!(truth.value, KernelValue::Number(LegacyNumber::Integer(1)));
    let error = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(0))],
            KernelExecutionBudget::default(),
        )
        .expect_err("Taken runtime numeric error");
    assert_eq!(
        error.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    let span = compiled
        .instruction_span(error.instruction.expect("Located runtime operation"))
        .expect("Original operation span");
    let source = SourceText::index(source, SourceBudget::default()).expect("Source");
    assert_eq!(source.span(span).expect("Mapped operator"), b"/");
    assert!(error.work_steps > 0);
    assert_eq!(error.admitted_local_values, 2);
}
