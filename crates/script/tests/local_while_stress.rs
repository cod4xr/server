// SPDX-License-Identifier: AGPL-3.0-or-later
//! Wide and deep loops avoid native short offsets, fixed masks and recursion.
#![allow(
    clippy::expect_used,
    reason = "Tests generate finite valid wide/deep scalar source."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};
use cod4x_script::execution::{KernelExecutionBudget, KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::syntax::LegacySyntaxBudget;
use std::fmt::Write;

#[test]
fn seventy_thousand_locals_and_long_backward_jumps_use_wide_instruction_indices() {
    let mut source = String::from("main(arg){");
    for index in 0..70_000 {
        write!(source, "v{index}=3;").expect("Generated wide source");
    }
    source.push_str("while(arg){");
    source.push_str(&"v69999=2;".repeat(40_000));
    source.push_str("arg--;}return v69999;}");
    // This combined wide-locals/large-body fixture separately admits its >1M CST nodes.
    let compiled = function_compile_support::compile_primitive_with_syntax_budget(
        source.as_bytes(),
        FunctionCompilationBudget::default(),
        LegacySyntaxBudget {
            max_nodes: 2_000_000,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Wide loop");
    assert_eq!(compiled.usage().local_bindings, 70_001);
    assert_eq!(compiled.usage().peak_control_bytes, 70_001);
    assert!(compiled.program().instructions().iter().enumerate().any(|(index, instruction)| matches!(instruction, KernelInstruction::Jump(target) if *target > 65_535 && index.saturating_sub(*target) > 65_535)));
    for (arg, expected) in [(0, 3), (1, 2)] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Wide loop invocation")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
}

#[test]
fn twelve_thousand_loops_lower_execute_and_fail_cleanup_without_recursive_ownership() {
    let source = format!(
        "main(arg){{{}return 1;{}return 2;}}",
        "while(arg){".repeat(12_000),
        "}".repeat(12_000)
    );
    let compiled = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget::default(),
    )
    .expect("Deep loops");
    assert_eq!(compiled.usage().peak_control_bytes, 12_000);
    for (arg, expected) in [(0, 2), (1, 1)] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Deep invocation")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
    let error = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget {
            max_control_bytes: 11_999,
            ..FunctionCompilationBudget::default()
        },
    )
    .expect_err("Deep quota failure");
    assert!(matches!(
        error.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::ControlBytes,
            requested: 12_000,
            maximum: 11_999
        }
    ));
    assert!(error.work_steps > 0);
}
