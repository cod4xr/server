// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests provide valid identifiers, source call forms and explicit unsupported profiles."
)]

use crate::function_compile_support;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompileErrorKind, ModuleCompilationBudget,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn bound_calls_use_legacy_case_identity_without_owning_source_names() {
    let module = module_compile_support::compile(
        b"main(){return HeLpEr();}helper(){return 17;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Case-folded local call");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Local identity")
            .value,
        KernelValue::Number(LegacyNumber::Integer(17))
    );
    let lower = "a".repeat(8192);
    let text = format!("main(){{return {lower}();}}{lower}(){{return 3;}}");
    let module = module_compile_support::compile(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Long names have storage rather than a native scratch cap");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Exact long identifier identity")
            .value,
        KernelValue::Number(LegacyNumber::Integer(3))
    );
    let text = format!(
        "main(){{return {}();}}{lower}(){{return 3;}}",
        lower.to_ascii_uppercase()
    );
    assert_eq!(
        module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default()
        )
        .expect_err("Native long-name fallback is case-sensitive")
        .cause
        .kind,
        FunctionCompileErrorKind::UnknownFunction
    );
}

#[test]
fn unresolved_external_receiver_thread_and_builtin_calls_fail_explicitly() {
    for (text, kind) in [
        (
            "main(){return unknown();}",
            FunctionCompileErrorKind::UnknownFunction,
        ),
        (
            "main(){return isdefined(1);}",
            FunctionCompileErrorKind::UnknownFunction,
        ),
        (
            "main(){return external::helper();}",
            FunctionCompileErrorKind::UnsupportedCall,
        ),
        (
            "main(){self helper();}helper(){}",
            FunctionCompileErrorKind::UnsupportedCall,
        ),
        (
            "main(){thread helper();}helper(){}",
            FunctionCompileErrorKind::UnsupportedCall,
        ),
    ] {
        let error = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect_err("Unsupported calls cannot gain ambient authority");
        assert_eq!(error.cause.kind, kind, "{text}");
        assert_eq!(error.declaration_index(), Some(0));
    }
    for text in [
        b"main(){return helper();}helper(){}".as_slice(),
        b"main(){helper();}helper(){}",
    ] {
        assert!(
            function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
                .is_err()
        );
        assert!(
            function_compile_support::compile_scalar(text, FunctionCompilationBudget::default())
                .is_err()
        );
    }
}
