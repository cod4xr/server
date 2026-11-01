// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise include-origin collisions and developer rules from bound source."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::compilation::{FunctionCompileErrorKind, ScriptPackageBudget};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn duplicate_unknown_and_colliding_includes_report_the_original_directive_privately() {
    for (source, kind) in [
        (
            b"#include lib;#include LIB;main(){return 0;}".as_slice(),
            FunctionCompileErrorKind::DuplicateInclude,
        ),
        (
            b"#include PrivateMissing;main(){return 0;}",
            FunctionCompileErrorKind::UnknownModule,
        ),
        (
            b"#include lib;main(){return 0;}work(){}",
            FunctionCompileErrorKind::IncludeFunctionConflict,
        ),
        (
            b"#include lib;#include other;main(){return 0;}",
            FunctionCompileErrorKind::IncludeFunctionConflict,
        ),
    ] {
        let error = compile(
            &[
                (b"PrivateRoot", source),
                (b"lib", b"work(){}"),
                (b"other", b"work(){}"),
            ],
            Disabled,
            ScriptPackageBudget::default(),
        )
        .expect_err("Every include/definition is checked");
        assert_eq!(error.module_index(), Some(0));
        assert_eq!(error.declaration_index(), None);
        assert!(error.cause.span.end > error.cause.span.start);
        assert_eq!(error.cause.kind, kind);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn imported_developer_targets_preserve_visibility_but_do_not_hide_builtin_fallback() {
    for (mode, kind) in [
        (Disabled, FunctionCompileErrorKind::UnknownFunction),
        (
            Enabled,
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ),
    ] {
        for source in [
            b"#include lib;main(){return hidden();}".as_slice(),
            b"#include lib;main(){return ::hidden;}",
        ] {
            let error = compile(
                &[(b"root", source), (b"lib", b"/# hidden(){return 7;}#/")],
                mode,
                ScriptPackageBudget::default(),
            )
            .expect_err("Ordinary caller cannot target developer code");
            assert_eq!(error.cause.kind, kind);
            assert_eq!(error.module_index(), Some(0));
            assert_eq!(error.declaration_index(), Some(0));
        }
    }
    for (source, expected) in [
        (b"#include lib;main(){return isint(1);}".as_slice(), 1),
        (b"#include lib;/# main(){return hidden();}#/", 7),
    ] {
        let package = compile(
            &[
                (b"root", source),
                (b"lib", b"/# hidden(){return 7;}isint(a){return 4;}#/"),
            ],
            Enabled,
            ScriptPackageBudget::default(),
        )
        .expect("Builtin/developer source rules");
        assert_eq!(
            package
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect("Permitted target")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
}

#[test]
fn forward_cyclic_includes_use_direct_exports_without_recursive_alias_expansion() {
    let package = compile(
        &[
            (
                b"a",
                b"#include b;main(){return work();}answer(){return 42;}",
            ),
            (b"b", b"#include a;work(){return answer();}"),
        ],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Cyclic source dependency graph");
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Original per-file scopes")
            .value,
        KernelValue::Number(LegacyNumber::Integer(42))
    );
    assert_eq!(package.usage().include_edges, 2);
    assert_eq!(package.usage().imported_functions, 3);
    let empty = compile(
        &[(b"empty", b"#include empty;")],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Empty self-include has no definitions to collide");
    assert!(empty.functions().is_empty());
    let error = compile(
        &[(b"a", b"#include a;main(){}")],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect_err("Nonempty self-include collides with its own export");
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::IncludeFunctionConflict
    );
}
