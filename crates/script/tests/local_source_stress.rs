// SPDX-License-Identifier: AGPL-3.0-or-later
//! Wide names/local IDs and deep source lowering do not retain native capacities.
#![allow(
    clippy::expect_used,
    reason = "Tests generate finite valid source within explicit default quotas."
)]

use std::fmt::Write;

use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};
use cod4x_script::execution::{KernelExecutionBudget, KernelInstruction, KernelValue};
use cod4x_script::legacy::LegacyNumber;

use crate::scalar_compile_support;

#[test]
fn seventy_thousand_source_locals_compile_and_execute_with_wide_slot_operands() {
    let mut source = String::from("main(){");
    for index in 0..70_000 {
        write!(source, "v{index}=1;").expect("Synthetic source growth");
    }
    source.push_str("return v69999;}");
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Wide local source compilation");
    assert_eq!(compiled.usage().local_bindings, 70_000);
    assert_eq!(compiled.program().layout().local_values(), 70_000);
    assert!(
        compiled
            .program()
            .instructions()
            .iter()
            .any(|instruction| matches!(instruction, KernelInstruction::LoadLocal(69_999)))
    );
    let outcome = compiled
        .program()
        .execute(KernelExecutionBudget::default())
        .expect("Wide source invocation");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(1)));
    assert_eq!(outcome.local_values, 70_000);
}

#[test]
fn hundred_thousand_byte_names_retain_legacy_exact_case_fallback_without_truncation() {
    let name = format!("P{}", "a".repeat(99_999));
    let source = format!("main(){{{name}=7;return {name};}}");
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Wide identifier compilation");
    assert_eq!(compiled.usage().local_bindings, 1);
    assert_eq!(compiled.usage().identifier_bytes, 300_000);
    assert_eq!(
        compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect("Wide name invocation")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
    let different = name.to_ascii_lowercase();
    let source = format!("main(){{{name}=7;return {different};}}");
    assert_eq!(
        scalar_compile_support::compile_local(
            source.as_bytes(),
            ScalarCompilationBudget::default()
        )
        .expect_err("Long names preserve exact case")
        .kind,
        ScalarCompileErrorKind::UninitializedLocal
    );
}

#[test]
fn twelve_thousand_groups_around_parameter_reads_lower_iteratively() {
    let source = format!(
        "main(arg){{x={}arg{};return x;}}",
        "(".repeat(12_000),
        ")".repeat(12_000)
    );
    let compiled = scalar_compile_support::compile_local(
        source.as_bytes(),
        ScalarCompilationBudget::default(),
    )
    .expect("Deep local source");
    let outcome = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(9))],
            KernelExecutionBudget::default(),
        )
        .expect("Deep local invocation");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(9)));
    assert_eq!(compiled.usage().local_bindings, 2);
}
