// SPDX-License-Identifier: AGPL-3.0-or-later
//! Transfer fixups/contexts retain wide indices and flat ownership under deep failure.
#![allow(
    clippy::expect_used,
    reason = "Tests generate finite valid transfer source."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};
use cod4x_script::execution::{KernelExecutionBudget, KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use std::fmt::Write;

#[test]
fn twenty_thousand_linked_break_fixups_resolve_wide_exits_without_native_case_arrays() {
    let mut source = String::from("main(arg){while(arg){");
    for index in 0..20_000 {
        write!(source, "if(arg=={index})break;").expect("Generated linked fixups");
    }
    source.push_str("arg=0;}return arg;}");
    let compiled = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget::default(),
    )
    .expect("Wide break fixup chain");
    let instructions = compiled.program().instructions();
    let jumps: Vec<_> = instructions
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
    for (arg, expected) in [(0, 0), (1, 1), (19_999, 19_999), (20_001, 0)] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Selected linked break")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
}

#[test]
fn twelve_thousand_nested_break_contexts_execute_and_fail_drop_iteratively() {
    let source = format!(
        "main(arg){{{}break;{}}}return 7;}}",
        "while(arg){if(arg==2)break;".repeat(12_000),
        "}break;".repeat(11_999)
    );
    let compiled = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget::default(),
    )
    .expect("Deep context ownership");
    assert_eq!(compiled.usage().peak_control_bytes, 12_001);
    for arg in [0, 1, 2] {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(
                    &[KernelValue::Number(LegacyNumber::Integer(arg))],
                    KernelExecutionBudget::default()
                )
                .expect("Nearest deep exits")
                .value,
            KernelValue::Number(LegacyNumber::Integer(7))
        );
    }
    let error = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget {
            max_control_bytes: 11_999,
            ..FunctionCompilationBudget::default()
        },
    )
    .expect_err("Deep failure drops saved outer contexts");
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
