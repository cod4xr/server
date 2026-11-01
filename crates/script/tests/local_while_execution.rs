// SPDX-License-Identifier: AGPL-3.0-or-later
//! While conditions reevaluate against private mutable scalar locals.
#![allow(
    clippy::expect_used,
    reason = "Tests invoke finite valid synthetic scalar loops."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::ScalarCompilationBudget;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue,
};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn repeated_and_nested_loops_reevaluate_mutation_and_lazy_conditions() {
    for (text, expected) in [
        (
            "main(count){sum=0;i=0;while(i<count){if(i%2==0){sum+=i;}else{sum-=i;}i++;}return sum;}",
            (0..20)
                .map(|n| (0..n).fold(0, |sum, i| if i % 2 == 0 { sum + i } else { sum - i }))
                .collect::<Vec<_>>(),
        ),
        (
            "main(count){i=0;total=0;while(i<count){j=0;while(j<i){total++;j++;}i++;}return total;}",
            (0..20).map(|n| n * (n - 1) / 2).collect(),
        ),
        (
            "main(count){i=0;while(i<count && !(i>=3)){i++;}return i;}",
            (0..20).map(|n| n.min(3)).collect(),
        ),
    ] {
        let mut source = text.as_bytes().to_vec();
        let compiled =
            scalar_compile_support::compile_local(&source, ScalarCompilationBudget::default())
                .expect("Loop source");
        source.fill(0);
        assert_eq!(compiled.usage().validation_steps, None);
        for (argument, expected) in (0..20).zip(expected) {
            let args = [KernelValue::Number(LegacyNumber::Integer(argument))];
            let result = compiled
                .program()
                .execute_with_arguments(&args, KernelExecutionBudget::default())
                .expect("Loop invocation");
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
fn zero_iterations_bare_returns_and_returning_bodies_have_real_results() {
    for (text, arg, expected) in [
        (
            "main(arg){while(arg)return 4;return 7;}",
            0,
            KernelValue::Number(LegacyNumber::Integer(7)),
        ),
        (
            "main(arg){while(arg)return 4;return 7;}",
            1,
            KernelValue::Number(LegacyNumber::Integer(4)),
        ),
        ("main(arg){while(arg){return;}}", 1, KernelValue::Undefined),
        ("main(arg){while(false){x=1/0;}}", 1, KernelValue::Undefined),
        (
            "main(arg){while(true){return 5;}}",
            0,
            KernelValue::Number(LegacyNumber::Integer(5)),
        ),
    ] {
        let compiled = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect("Return/empty loop");
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Loop result")
                .value,
            expected
        );
    }
}

#[test]
fn loop_condition_body_and_fuel_errors_remain_located_and_bounded() {
    let text = b"PrivateMain(arg){while(arg){x=1/0;}return 7;}";
    let compiled = scalar_compile_support::compile_local(text, ScalarCompilationBudget::default())
        .expect("Runtime-dependent loop");
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    for (arg, kind, token) in [
        (
            KernelValue::Undefined,
            KernelErrorKind::ExpectedNumber,
            b"while".as_slice(),
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
            .expect_err("Runtime error");
        assert_eq!(error.kind, kind);
        let span = compiled
            .instruction_span(error.instruction.expect("Located PC"))
            .expect("Source map");
        assert_eq!(source.span(span).expect("Original token"), token);
        assert!(!format!("{error:?} {compiled:?}").contains("Private"));
    }
    let infinite = scalar_compile_support::compile_local(
        b"main(arg){while(arg){}}",
        ScalarCompilationBudget::default(),
    )
    .expect("Fuel-bounded source");
    let error = infinite
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelExecutionBudget {
                max_steps: 50,
                ..KernelExecutionBudget::default()
            },
        )
        .expect_err("Deterministic fuel exhaustion");
    assert_eq!(error.work_steps, 50);
    assert_eq!(error.admitted_local_values, 1);
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            requested: 51,
            maximum: 50
        }
    ));
}
