// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid bound declarations under explicit developer modes."
)]

use crate::module_compile_support;
use cod4x_script::binding::{
    LegacyDeveloperMode, LegacyFunctionInclusion, LegacyFunctionVisibility,
};
use cod4x_script::compilation::{
    FunctionCompileErrorKind, ModuleCompilationBudget, ModuleCompilationUsage,
};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelModuleVerificationBudget, KernelValue,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn disabled_unsupported_bodies_remain_metadata_without_executable_placeholders() {
    let text = b"main(){return 1;}/# PrivateHidden(){return \"PrivateUnsupported\";}#/";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Disabled unsupported body stays inactive");
    assert_eq!(module.declarations().len(), 2);
    assert_eq!(module.functions().len(), 1);
    assert_eq!(module.function_for_declaration(1), None);
    assert_eq!(
        module.declarations()[1].visibility,
        LegacyFunctionVisibility::DeveloperOnly
    );
    assert_eq!(
        module.declarations()[1].inclusion,
        LegacyFunctionInclusion::DeveloperDisabled
    );
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Ordinary code remains usable")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    let error = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Enabled,
        ModuleCompilationBudget::default(),
    )
    .expect_err("Active unsupported body prevents complete publication");
    assert_eq!(error.declaration_index(), Some(1));
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::UnsupportedPrimary
    );
    assert!(!format!("{error:?} {error}").contains("Private"));
}

#[test]
fn ordinary_references_to_developer_targets_are_rejected_separately_from_inclusion() {
    let text = b"main(){return hidden();}/# hidden(){return 7;}#/";
    for (mode, kind) in [
        (
            LegacyDeveloperMode::Disabled,
            FunctionCompileErrorKind::UnknownFunction,
        ),
        (
            LegacyDeveloperMode::Enabled,
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ),
    ] {
        let error = module_compile_support::compile(text, mode, ModuleCompilationBudget::default())
            .expect_err("No ordinary-to-developer reference");
        assert_eq!(error.declaration_index(), Some(0));
        assert_eq!(error.cause.kind, kind);
    }
    let text = b"/# devmain(){return helper();}#/ helper(){return 7;}";
    let enabled = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Enabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Developer caller may reference ordinary target");
    assert_eq!(
        enabled
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Developer entry selected explicitly")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
    let disabled = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Original declaration order survives disabled code");
    assert_eq!(disabled.function_for_declaration(0), None);
    assert_eq!(disabled.function_for_declaration(1), Some(0));
    let text = b"/# main(){return helper();}helper(){return 8;}#/";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Enabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Developer-to-developer references remain active");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Developer call")
            .value,
        KernelValue::Number(LegacyNumber::Integer(8))
    );
}

#[test]
fn empty_and_all_disabled_modules_preserve_metadata_without_fabricated_functions() {
    let zero = ModuleCompilationBudget {
        max_declarations: 0,
        max_functions: 0,
        max_steps: 0,
        max_instructions: 0,
        max_metadata_bytes: 0,
        verification: KernelModuleVerificationBudget {
            max_functions: 0,
            max_instructions: 0,
            max_steps: 0,
            max_metadata_bytes: 0,
            max_function_stack_values: 0,
            max_function_local_values: 0,
        },
        ..ModuleCompilationBudget::default()
    };
    let empty = module_compile_support::compile(b"", LegacyDeveloperMode::Disabled, zero)
        .expect("Empty module fits zero admission");
    assert_eq!(empty.usage(), ModuleCompilationUsage::default());
    assert!(empty.declarations().is_empty() && empty.functions().is_empty());
    let disabled = module_compile_support::compile(
        b"/# hidden(){return \"unsupported\";}#/",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("All-disabled module retains declarations");
    assert_eq!(disabled.declarations().len(), 1);
    assert_eq!(disabled.program().usage().functions, 0);
    assert_eq!(disabled.usage().instructions, 0);
    for module in [empty, disabled] {
        assert!(module.functions().is_empty());
        assert_eq!(
            module
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect_err("No entry exists")
                .cause
                .kind,
            KernelErrorKind::InvalidFunction { function: 0 }
        );
    }
}
