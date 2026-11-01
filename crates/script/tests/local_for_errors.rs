// SPDX-License-Identifier: AGPL-3.0-or-later
//! Clause failures and deferred jumps preserve original spans and instruction fuel.
#![allow(
    clippy::expect_used,
    reason = "Tests invoke finite or explicitly fuel-bounded source."
)]

use crate::function_compile_support;
use cod4x_script::compilation::FunctionCompilationBudget;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue,
};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn initializer_condition_and_update_errors_map_to_their_original_operations() {
    for (text, arg, kind, token) in [
        (
            "PrivateMain(arg){for(i=arg/0;false;){}}",
            KernelValue::Number(LegacyNumber::Integer(1)),
            KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero),
            "/",
        ),
        (
            "PrivateMain(arg){for(;arg;){break;}}",
            KernelValue::Undefined,
            KernelErrorKind::ExpectedNumber,
            "for",
        ),
        (
            "PrivateMain(arg){for(i=0;i<1;arg++){i++;continue;}return arg;}",
            KernelValue::Number(LegacyNumber::Float(1.5)),
            KernelErrorKind::Numeric(LegacyNumberError::ExpectedInteger),
            "++",
        ),
        (
            "PrivateMain(arg){for(i=0;i<1;i/=0){}}",
            KernelValue::Undefined,
            KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero),
            "/=",
        ),
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Runtime-dependent clauses");
        let args = [arg];
        let error = compiled
            .program()
            .execute_with_arguments(&args, KernelExecutionBudget::default())
            .expect_err("Located clause failure");
        assert_eq!(error.kind, kind);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source
                .span(
                    compiled
                        .instruction_span(error.instruction.expect("PC"))
                        .expect("Map")
                )
                .expect("Token"),
            token.as_bytes()
        );
        assert_eq!(args[0], arg);
        assert!(error.work_steps > 0);
        assert!(!format!("{compiled:?} {error:?}").contains("Private"));
    }
}

#[test]
fn omitted_condition_and_deferred_continue_cannot_evade_fuel() {
    let text = b"main(){for(;;){continue;}}";
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Admitted endless source");
    let error = compiled
        .program()
        .execute(KernelExecutionBudget {
            max_steps: 3,
            ..KernelExecutionBudget::default()
        })
        .expect_err("Fuel stops before the backward jump");
    assert_eq!(error.work_steps, 3);
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            requested: 4,
            maximum: 3
        }
    ));
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(
        source
            .span(
                compiled
                    .instruction_span(error.instruction.expect("PC"))
                    .expect("Map")
            )
            .expect("Loop keyword"),
        b"for"
    );
}
