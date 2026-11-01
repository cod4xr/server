// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise package visibility and sanitized located failures from real source."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::compilation::{FunctionCompileErrorKind, ScriptPackageBudget};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn qualified_calls_enforce_developer_visibility_and_disabled_bodies_remain_metadata() {
    for (mode, expected) in [
        (Disabled, FunctionCompileErrorKind::UnknownFunction),
        (
            Enabled,
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ),
    ] {
        for source in [
            b"main(){return lib::hidden();}".as_slice(),
            b"main(){return lib::hidden;}",
        ] {
            let error = compile(
                &[(b"root", source), (b"lib", b"/# hidden(){return 7;}#/")],
                mode,
                ScriptPackageBudget::default(),
            )
            .expect_err("Ordinary caller cannot target developer code");
            assert_eq!(error.module_index(), Some(0));
            assert_eq!(error.declaration_index(), Some(0));
            assert_eq!(error.cause.kind, expected);
        }
    }
    let package = compile(
        &[
            (b"empty", b""),
            (b"root", b"/# main(){return lib::helper();}#/"),
            (b"middle", b""),
            (b"lib", b"helper(){return 9;}"),
            (b"last", b""),
        ],
        Enabled,
        ScriptPackageBudget::default(),
    )
    .expect("Developer caller may target ordinary code");
    assert_eq!(
        package
            .instruction_location(0, 0)
            .expect("Root location")
            .module,
        1
    );
    assert_eq!(
        package
            .instruction_location(1, 0)
            .expect("Library location")
            .module,
        3
    );
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Developer call")
            .value,
        KernelValue::Number(LegacyNumber::Integer(9))
    );
    let inactive = compile(
        &[(b"lib", b"/# hidden(){return self;}#/"), (b"empty", b"")],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Unsupported disabled body is not lowered");
    assert_eq!(inactive.declarations().len(), 1);
    assert!(inactive.functions().is_empty());
    assert_eq!(inactive.function_for_declaration(0, 0), None);
    assert_eq!(inactive.instruction_location(0, 0), None);
}

#[test]
fn qualified_failures_never_fall_back_to_builtins_or_expose_names() {
    for (source, kind) in [
        (
            b"main(){return PrivateMissing::isint(1);}".as_slice(),
            FunctionCompileErrorKind::UnknownModule,
        ),
        (
            b"main(){return PrivateLibrary::isint(1);}",
            FunctionCompileErrorKind::UnknownFunction,
        ),
        (
            b"main(){return PrivateLibrary::isint;}",
            FunctionCompileErrorKind::UnknownFunction,
        ),
    ] {
        let error = compile(
            &[
                (b"PrivateRoot", source),
                (b"PrivateLibrary", b"other(){return 0;}"),
            ],
            Disabled,
            ScriptPackageBudget::default(),
        )
        .expect_err("Qualified misses are strict");
        assert_eq!(error.cause.kind, kind);
        assert_eq!(error.module_index(), Some(0));
        assert_eq!(error.declaration_index(), Some(0));
        assert!(error.cause.span.end > error.cause.span.start);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
    let package = compile(
        &[
            (
                b"PrivateRoot",
                b"main(){return isint(1)+lib::isint(1);}isint(a){return 4;}",
            ),
            (b"lib", b"isint(a){return 5;}"),
        ],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Each file's script definition precedes builtins");
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Shadowed names")
            .value,
        KernelValue::Number(LegacyNumber::Integer(9))
    );
    assert!(!format!("{package:?}").contains("Private"));
}

#[test]
fn later_file_lowering_and_proof_failures_map_to_original_local_declaration() {
    let inputs = [
        (b"a".as_slice(), b"main(){return 0;}".as_slice()),
        (b"empty", b""),
        (b"b", b"first(){return 1;}later(x){return x;}"),
    ];
    let mut budget = ScriptPackageBudget::default();
    budget.compilation.verification.max_function_local_values = 0;
    let error = compile(&inputs, Disabled, budget).expect_err("Later proof layout failure");
    assert_eq!(error.module_index(), Some(2));
    assert_eq!(error.declaration_index(), Some(1));
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::Kernel(_)
    ));
    let error = compile(
        &[
            (b"a", b"main(){return 0;}"),
            (b"b", b"first(){return 1;}later(){return missing();}"),
        ],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect_err("Later lowering failure");
    assert_eq!(error.module_index(), Some(1));
    assert_eq!(error.declaration_index(), Some(1));
    assert_eq!(error.cause.kind, FunctionCompileErrorKind::UnknownFunction);
}
