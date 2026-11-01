// SPDX-License-Identifier: AGPL-3.0-or-later
//! While transfers target the nearest active loop and terminate structural paths.
#![allow(
    clippy::expect_used,
    reason = "Tests compile finite synthetic scalar transfer programs."
)]

use crate::function_compile_support;
use cod4x_script::compilation::FunctionCompilationBudget;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn break_continue_nested_contexts_and_reaching_joins_execute() {
    for (text, results) in [
        (
            "main(count){i=0;sum=0;while(i<count){i++;if(i==2)continue;if(i>5)break;sum+=i;}return sum;}",
            (0..9)
                .map(|n| (1..=n.min(5)).filter(|i| *i != 2).sum())
                .collect::<Vec<i32>>(),
        ),
        (
            "main(count){i=0;sum=0;while(i<count){i++;j=0;while(j<10){j++;if(j==2)continue;if(j==4)break;sum+=j;}if(i==2)continue;if(i>=4)break;sum+=100;}return sum;}",
            (0..9)
                .map(|n| {
                    (1..=n.min(4))
                        .map(|i| 4 + if i == 2 || i == 4 { 0 } else { 100 })
                        .sum()
                })
                .collect(),
        ),
        (
            "main(arg){while(arg){if(arg==1)break;else return 9;}return 7;}",
            vec![7, 7, 9, 9, 9, 9, 9, 9, 9],
        ),
        (
            "main(arg){while(arg){if(arg>1){arg--;continue;}else{x=3;}return x;}return 7;}",
            vec![7, 3, 3, 3, 3, 3, 3, 3, 3],
        ),
        (
            "main(arg){while(arg){if(arg==1){arg--;continue;}else{arg--;continue;}}return arg;}",
            vec![0; 9],
        ),
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Scoped transfer source");
        for (arg, expected) in (0..9).zip(results) {
            let args = [KernelValue::Number(LegacyNumber::Integer(arg))];
            assert_eq!(
                compiled
                    .program()
                    .execute_with_arguments(&args, KernelExecutionBudget::default())
                    .expect("Scoped transfer invocation")
                    .value,
                KernelValue::Number(LegacyNumber::Integer(expected)),
                "{text}"
            );
            assert_eq!(args[0], KernelValue::Number(LegacyNumber::Integer(arg)));
        }
    }
}

#[test]
fn continue_fuel_and_original_keyword_maps_remain_bounded_and_private() {
    let text = b"PrivateMain(arg){while(arg){continue;}}";
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Fuel-bounded continue source");
    // Load + branch + continue repeats; after five instructions PC names continue.
    let error = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelExecutionBudget {
                max_steps: 5,
                ..KernelExecutionBudget::default()
            },
        )
        .expect_err("Continue cannot evade fuel");
    assert_eq!(error.work_steps, 5);
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            ..
        }
    ));
    let span = compiled
        .instruction_span(error.instruction.expect("Stopped PC"))
        .expect("Original map");
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(source.span(span).expect("Transfer token"), b"continue");
    assert!(!format!("{compiled:?} {error:?}").contains("Private"));
    let text = b"main(arg){while(arg){break;}return 3;}";
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Break source map");
    let error = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelExecutionBudget {
                max_steps: 2,
                ..KernelExecutionBudget::default()
            },
        )
        .expect_err("Fuel stops before break jump");
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(
        source
            .span(
                compiled
                    .instruction_span(error.instruction.expect("PC"))
                    .expect("Map")
            )
            .expect("Token"),
        b"break"
    );
}
