// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deep lowering/drop and large real branch destinations remain iterative and wide.
#![allow(
    clippy::expect_used,
    reason = "Tests construct finite valid source within explicit default budgets."
)]

use cod4x_script::compilation::ScalarCompilationBudget;
use cod4x_script::execution::{KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

use crate::scalar_compile_support;

#[test]
fn twelve_thousand_groups_and_left_operators_compile_execute_and_drop_without_recursion() {
    let count = 12_000;
    let grouped = format!(
        "main(){{return {}1{};}}",
        "(".repeat(count),
        ")".repeat(count)
    );
    let chained = format!("main(){{return {}1;}}", "1+".repeat(count));
    for (text, expected) in [(grouped, 1), (chained, 12_001)] {
        let compiled =
            scalar_compile_support::compile(text.as_bytes(), ScalarCompilationBudget::default())
                .expect("Deep scalar source");
        let outcome = compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect("Deep scalar execution");
        assert_eq!(
            outcome.value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
        assert!(compiled.usage().compiler_steps < 1_000_000);
    }
}

#[test]
fn logical_source_resolves_branch_destinations_beyond_sixteen_bits() {
    let text = format!("main(){{return {}true;}}", "false||".repeat(12_000));
    let compiled =
        scalar_compile_support::compile(text.as_bytes(), ScalarCompilationBudget::default())
            .expect("Large logical source");
    assert!(compiled.usage().instructions > u64::from(u16::MAX));
    let outcome = compiled
        .program()
        .execute(KernelExecutionBudget::default())
        .expect("Wide logical execution");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(1)));
    assert_eq!(outcome.steps, compiled.usage().validation_steps);
}
