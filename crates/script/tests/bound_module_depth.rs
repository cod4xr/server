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
    deep_calls(Invocation::Direct);
    deep_calls(Invocation::Indirect);
    deep_calls(Invocation::PureBuiltin);
}

#[derive(Clone, Copy)]
enum Invocation {
    Direct,
    Indirect,
    PureBuiltin,
}

fn deep_calls(invocation: Invocation) {
    let mut text = String::with_capacity(50_000);
    text.push_str("main(){return ");
    for _ in 0..12_000 {
        text.push_str(match invocation {
            Invocation::Direct => "id(",
            Invocation::Indirect => "[[::id]](",
            Invocation::PureBuiltin => "isint(",
        });
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
    assert_eq!(
        outcome.usage.steps,
        match invocation {
            Invocation::Direct => 36_002,
            Invocation::Indirect => 48_002,
            Invocation::PureBuiltin => 12_002,
        }
    );
    assert_eq!(
        outcome.usage.peak_frames,
        match invocation {
            Invocation::PureBuiltin => 1,
            _ => 2,
        }
    );
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
