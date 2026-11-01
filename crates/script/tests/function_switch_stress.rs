// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite wide/deep source explicitly admits frontend/compiler scopes."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationResource, FunctionCompileErrorKind, ModuleCompilationBudget,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::syntax::LegacySyntaxBudget;
use std::fmt::Write;

#[test]
fn sixty_six_thousand_cases_keep_wide_addresses_and_logarithmic_dispatch() {
    let mut text = String::with_capacity(2_000_000);
    text.push_str("main(n){switch(n){");
    for key in (0..66_000).rev() {
        write!(&mut text, "case {key}:return {key};").expect("Synthetic label");
    }
    text.push_str("default:return -1;}}");
    let module = module_compile_support::compile_with_syntax_budget(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
        LegacySyntaxBudget {
            max_nodes: 4_000_000,
            max_child_edges: 5_000_000,
            max_metadata_bytes: 512 * 1024 * 1024,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Case count exceeds the native short inventory");
    for key in [0, 1, 32_767, 65_535, 65_999, 66_000, -1] {
        let result = module
            .program()
            .execute(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(key))],
                KernelCallBudget::default(),
            )
            .expect("Wide balanced case lookup");
        assert_eq!(
            result.value,
            KernelValue::Number(LegacyNumber::Integer(if (0..66_000).contains(&key) {
                key
            } else {
                -1
            }))
        );
        assert!(
            result.usage.steps <= 195,
            "Tree selection must not scan all cases"
        );
    }
    assert!(module.functions()[0].usage().instructions > 65_535);
}

#[test]
fn twelve_thousand_nested_switches_compile_execute_and_fail_cleanup_flatly() {
    let mut text = String::with_capacity(400_000);
    text.push_str("main(n){");
    for _ in 0..12_000 {
        text.push_str("switch(n){default:");
    }
    text.push_str("return 7;");
    for _ in 0..12_000 {
        text.push('}');
    }
    text.push('}');
    let module = module_compile_support::compile(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Flat switch context ownership");
    let result = module
        .program()
        .execute(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(0))],
            KernelCallBudget::default(),
        )
        .expect("Flat typed control flow");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(7)));
    assert_eq!(result.usage.steps, 60_002);
    assert_eq!(result.usage.peak_frames, 1);
    let mut budget = ModuleCompilationBudget::default();
    budget.function.max_task_frames = 128;
    let error =
        module_compile_support::compile(text.as_bytes(), LegacyDeveloperMode::Disabled, budget)
            .expect_err("Deep failure cleanup remains nonrecursive");
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::TaskFrames,
            ..
        }
    ));
}
