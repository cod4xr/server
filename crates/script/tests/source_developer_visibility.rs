// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Developer list visibility must not escape into ordinary source or admit nested developer execution."
)]
use crate::{
    array_read_support::number, module_compile_support, source_developer_support::compile,
};
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::KernelCallBudget;

#[test]
fn developer_calls_and_function_references_gain_visibility_only_inside_their_active_region() {
    let source = b"main(){/# f=::hidden;return [[f]](); #/return 9;}/# hidden(){return 42;} #/";
    for (mode, expected) in [(Enabled, 42), (Disabled, 9)] {
        let module = compile(source, mode);
        assert_eq!(
            module
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect("Developer function binding")
                .value,
            number(expected)
        );
    }
    let outside = b"main(){/# f=::hidden; #/return hidden();}/# hidden(){return 42;} #/";
    let error =
        module_compile_support::compile(outside, Enabled, ModuleCompilationBudget::default())
            .expect_err("Ordinary visibility restored");
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::DeveloperFunctionReference
    );
}

#[test]
fn nested_developer_lists_and_lists_inside_developer_only_definitions_fail_privately() {
    for source in [
        b"main(){/# /# x=1; #/ #/return 0;}".as_slice(),
        b"main(){return 0;}/# PrivateHidden(){/# return 1; #/return 0;} #/",
    ] {
        for mode in [Enabled, Disabled] {
            // Disabled top-level definitions remain excluded from compilation.
            if mode == Disabled && source.starts_with(b"main(){return") {
                continue;
            }
            let error =
                module_compile_support::compile(source, mode, ModuleCompilationBudget::default())
                    .expect_err("Nested developer region");
            assert_eq!(
                error.cause.kind,
                FunctionCompileErrorKind::NestedDeveloperBlock
            );
            assert!(!format!("{error:?} {error}").contains("Private"));
        }
    }
}
