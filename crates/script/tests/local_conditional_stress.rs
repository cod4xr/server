// SPDX-License-Identifier: AGPL-3.0-or-later
//! Control flow uses wide slots/targets and flat lowering/drop at deep nesting.
#![allow(
    clippy::expect_used,
    reason = "Tests generate finite valid wide/deep source."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompileErrorKind,
};
use cod4x_script::execution::{KernelExecutionBudget, KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use std::fmt::Write;

#[test]
fn seventy_thousand_local_snapshots_keep_wide_slots_and_targets() {
    let mut source = String::from("main(arg){");
    for index in 0..70_000 {
        write!(source, "v{index}=1;").expect("Generated source");
    }
    source.push_str("if(arg){v69999=2;}else{v69999=3;}return v69999;}");
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Wide facts");
    assert_eq!(compiled.usage().local_bindings, 70_001);
    assert_eq!(compiled.usage().peak_control_bytes, 140_002);
    assert!(compiled.program().instructions().iter().any(|instruction| matches!(instruction, KernelInstruction::Branch { target, .. } if *target > 65_535)));
    for (arg, expected) in [(0, 3), (1, 2)] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Wide invocation")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
}

#[test]
fn large_forward_branch_skips_more_than_sixty_five_thousand_instructions() {
    let source = format!(
        "main(arg){{x=0;if(arg){{{}}}return x;}}",
        "x=1;".repeat(40_000)
    );
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Wide forward destination");
    assert!(compiled.program().instructions().iter().enumerate().any(|(index, instruction)| matches!(instruction, KernelInstruction::Branch { target, .. } if target.saturating_sub(index) > 65_535)));
    let skipped = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(0))],
            KernelExecutionBudget::default(),
        )
        .expect("Skip wide block");
    assert_eq!(skipped.value, KernelValue::Number(LegacyNumber::Integer(0)));
    assert!(skipped.steps < 10);
}

#[test]
fn twelve_thousand_nested_branches_lower_execute_and_fail_drop_iteratively() {
    let source = format!(
        "main(arg){{x=1;{}x=2;{}return x;}}",
        "if(arg){".repeat(12_000),
        "}".repeat(12_000)
    );
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Deep iterative source");
    assert_eq!(compiled.usage().peak_control_bytes, 24_000);
    for (arg, expected) in [(0, 1), (1, 2)] {
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
    let error = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget {
            max_control_bytes: 23_999,
            ..ScalarCompilationBudget::default()
        },
    )
    .expect_err("Deep failure drops flat pending tasks");
    assert!(matches!(
        error.kind,
        ScalarCompileErrorKind::BudgetExceeded {
            resource: ScalarCompilationResource::ControlBytes,
            ..
        }
    ));
    assert!(error.work_steps > 0);
}
