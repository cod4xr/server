// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite deep source is admitted explicitly and exercises flat completion/failure."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationResource, FunctionCompileErrorKind, ModuleCompilationBudget,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn twelve_thousand_nested_source_calls_lower_and_fail_without_host_recursion() {
    deep_calls(false);
    deep_calls(true);
}

fn deep_calls(indirect: bool) {
    let mut text = String::with_capacity(50_000);
    text.push_str("main(){return ");
    for _ in 0..12_000 {
        text.push_str(if indirect { "[[::id]](" } else { "id(" });
    }
    text.push('1');
    for _ in 0..12_000 {
        text.push(')');
    }
    text.push_str(";}id(x){return x;}");
    let module = module_compile_support::compile(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Flat source-call lowering");
    let outcome = module
        .program()
        .execute(0, &[], KernelCallBudget::default())
        .expect("Innermost-first nested call execution");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(1)));
    assert_eq!(outcome.usage.steps, if indirect { 48_002 } else { 36_002 });
    assert_eq!(outcome.usage.peak_frames, 2);
    assert!(module.functions()[0].usage().peak_task_frames >= 12_000);
    let mut budget = ModuleCompilationBudget::default();
    budget.function.max_task_frames = 128;
    let error =
        module_compile_support::compile(text.as_bytes(), LegacyDeveloperMode::Disabled, budget)
            .expect_err("Deep failed cleanup remains flat");
    assert_eq!(error.declaration_index(), Some(0));
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::TaskFrames,
            ..
        }
    ));
}
