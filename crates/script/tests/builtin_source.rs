// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile finite pure builtin source cases."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn pure_predicates_execute_in_source_and_keep_legacy_case_identity() {
    for (text, expected) in [
        ("main(){return IsInT(0)+isfloat(0.0)+isvector((1,2,3));}", 3),
        (
            "main(){return isint(undefined)+isfloat(::main)+isvector(7);}",
            0,
        ),
        (
            "main(){isint(1);return isfloat(1)+isvector(1.0)+isint((1,2,3));}",
            0,
        ),
        ("main(){return wrap(7);}wrap(n){return isint(n);}", 1),
    ] {
        let module = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Pure builtin source");
        let result = module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Builtin result");
        assert_eq!(
            result.value,
            KernelValue::Number(LegacyNumber::Integer(expected)),
            "{text}"
        );
    }
}

#[test]
fn local_definitions_shadow_builtins_and_developer_policy_cannot_fall_back() {
    for text in [
        b"main(){return IsInT(7);}isint(n){return 23;}".as_slice(),
        b"main(){return [[::isint]](7);}isint(n){return 23;}",
    ] {
        let module = module_compile_support::compile(
            text,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Local script shadows builtin");
        assert_eq!(
            module
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect("Local result")
                .value,
            KernelValue::Number(LegacyNumber::Integer(23))
        );
    }
    let text = b"main(){return isint(7);}/# isint(n){return 23;} #/";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Inactive declaration does not shadow");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Builtin result")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    assert_eq!(
        module_compile_support::compile(
            text,
            LegacyDeveloperMode::Enabled,
            ModuleCompilationBudget::default()
        )
        .expect_err("Active developer target cannot fall back")
        .cause
        .kind,
        FunctionCompileErrorKind::DeveloperFunctionReference
    );
}

#[test]
fn builtin_calls_gain_no_reference_external_receiver_thread_or_unknown_authority() {
    for (text, expected) in [
        (
            "main(){return ::isint;}",
            FunctionCompileErrorKind::UnknownFunction,
        ),
        (
            "main(){return external::isint(1);}",
            FunctionCompileErrorKind::UnsupportedCall,
        ),
        (
            "main(){self isint(1);}",
            FunctionCompileErrorKind::UnsupportedCall,
        ),
        (
            "main(){thread isint(1);}",
            FunctionCompileErrorKind::UnsupportedCall,
        ),
        (
            "main(){return openfile(1);}",
            FunctionCompileErrorKind::UnknownFunction,
        ),
    ] {
        assert_eq!(
            module_compile_support::compile(
                text.as_bytes(),
                LegacyDeveloperMode::Disabled,
                ModuleCompilationBudget::default()
            )
            .expect_err("No ambient authority")
            .cause
            .kind,
            expected
        );
    }
}
