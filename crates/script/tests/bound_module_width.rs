// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite synthetic sources explicitly admit large frontend and compiler scopes."
)]

use std::fmt::Write;

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompileErrorKind, ModuleCompilationBudget, ModuleCompilationResource,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::syntax::LegacySyntaxBudget;

fn wide_syntax() -> LegacySyntaxBudget {
    LegacySyntaxBudget {
        max_nodes: 4_000_000,
        max_child_edges: 5_000_000,
        max_metadata_bytes: 512 * 1024 * 1024,
        ..LegacySyntaxBudget::default()
    }
}

#[test]
fn seventy_thousand_source_arguments_and_parameters_compile_and_execute_without_short_indices() {
    let mut text = String::with_capacity(1_000_000);
    text.push_str("main(){return sink(");
    for index in 0..70_000 {
        if index > 0 {
            text.push(',');
        }
        write!(&mut text, "{index}").expect("Synthetic literal");
    }
    text.push_str(");}sink(");
    for index in 0..70_000 {
        if index > 0 {
            text.push(',');
        }
        write!(&mut text, "p{index}").expect("Synthetic parameter");
    }
    text.push_str("){return p69999;}");
    let module = module_compile_support::compile_with_syntax_budget(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
        wide_syntax(),
    )
    .expect("Wide real source calls");
    let outcome = module
        .program()
        .execute(0, &[], KernelCallBudget::default())
        .expect("Wide source parameter binding");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(69_999))
    );
    assert_eq!(outcome.usage.steps, 70_004);
    assert_eq!(outcome.usage.peak_local_values, 70_000);
    assert_eq!(outcome.usage.peak_stack_values, 70_001);
    let mut budget = ModuleCompilationBudget::default();
    budget.function.max_call_arguments = 69_999;
    let error = module_compile_support::compile_with_syntax_budget(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        budget,
        wide_syntax(),
    )
    .expect_err("Wide count preflight fails before operand lowering");
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::ModuleBudgetExceeded {
            resource: ModuleCompilationResource::CallArguments,
            requested: 70_000,
            maximum: 69_999
        }
    ));
}

#[test]
fn source_linking_preserves_sixty_six_thousand_distinct_function_targets_and_maps() {
    let mut text = String::with_capacity(1_600_000);
    text.push_str("main(){return f65999();}");
    for index in 1..66_000 {
        let result = i32::from(index == 65_999) * 42;
        write!(&mut text, "f{index}(){{return {result};}}").expect("Synthetic function");
    }
    let module = module_compile_support::compile_with_syntax_budget(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
        wide_syntax(),
    )
    .expect("Wide source inventory");
    assert_eq!(module.functions().len(), 66_000);
    assert_eq!(module.declarations().len(), 66_000);
    assert_eq!(module.function_for_declaration(65_999), Some(65_999));
    assert_eq!(module.functions()[65_999].declaration_index(), 65_999);
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Forward call to wide final target")
            .value,
        KernelValue::Number(LegacyNumber::Integer(42))
    );
    assert!(module.instruction_span(65_999, 0).is_some());
}
