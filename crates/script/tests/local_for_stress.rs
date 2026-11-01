// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deferred continue fixups and nested for contexts retain wide flat ownership.
#![allow(
    clippy::expect_used,
    reason = "Tests generate finite valid deep/wide source."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompileErrorKind,
};
use cod4x_script::execution::{KernelExecutionBudget, KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use std::fmt::Write;

#[test]
fn twenty_thousand_continue_fixups_reach_a_wide_update_destination() {
    let mut source = String::from("main(arg){for(i=0;i<1;i++){");
    for index in 0..20_000 {
        write!(source, "if(arg=={index})continue;").expect("Generated deferred fixups");
    }
    source.push_str("}return i;}");
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Wide deferred fixups");
    let jumps: Vec<_> = compiled
        .program()
        .instructions()
        .iter()
        .enumerate()
        .filter_map(|(index, instruction)| match instruction {
            KernelInstruction::Jump(target) if *target > index => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(jumps.len(), 20_000);
    assert!(
        jumps
            .iter()
            .all(|target| *target == jumps[0] && *target > 65_535)
    );
    for arg in [0, 1, 19_999, 20_001] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Continue runs update")
                .value,
            KernelValue::Number(LegacyNumber::Integer(1))
        );
    }
}

#[test]
fn twelve_thousand_for_contexts_execute_and_fail_cleanup_iteratively() {
    let source = format!(
        "main(arg){{{}break;{}}}return 7;}}",
        "for(;arg;){if(arg==2)break;".repeat(12_000),
        "}break;".repeat(11_999)
    );
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Deep for contexts");
    assert_eq!(compiled.usage().peak_control_bytes, 12_001);
    for arg in [0, 1, 2] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Deep nearest-loop exits")
                .value,
            KernelValue::Number(LegacyNumber::Integer(7))
        );
    }
    let error = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget {
            max_control_bytes: 11_999,
            ..ScalarCompilationBudget::default()
        },
    )
    .expect_err("Deep bounded failure");
    assert!(matches!(
        error.kind,
        ScalarCompileErrorKind::BudgetExceeded {
            resource: ScalarCompilationResource::ControlBytes,
            requested: 12_000,
            maximum: 11_999
        }
    ));
    assert!(error.work_steps > 0);
}
