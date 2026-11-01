// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute explicit one-hop includes while retaining original file namespaces."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::{FunctionCompileErrorKind, ScriptPackageBudget};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn included_calls_and_function_references_keep_the_original_body_namespace() {
    let package = compile(
        &[
            (
                b"root",
                b"#include lib;main(){f=::work;return work()+[[f]]();}",
            ),
            (b"lib", b"work(){return helper();}helper(){return 21;}"),
        ],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("One-hop imports");
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Direct/indirect imported calls")
            .value,
        KernelValue::Number(LegacyNumber::Integer(42))
    );
    assert_eq!(
        package
            .instruction_location(1, 0)
            .expect("Original file")
            .module,
        1
    );
    assert_eq!(package.functions().len(), 3);
    assert_eq!(package.usage().include_edges, 1);
    assert_eq!(package.usage().imported_functions, 2);
}

#[test]
fn imported_function_references_and_builtin_calls_have_distinct_resolution() {
    let package = compile(
        &[
            (
                b"root",
                b"#include lib;main(){f=::isint;return isint(1)+[[f]](1);}",
            ),
            (b"lib", b"isint(a){return 4;}"),
        ],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Builtin/deferred import distinction");
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Named builtin plus script pointer")
            .value,
        KernelValue::Number(LegacyNumber::Integer(5))
    );
}

#[test]
fn imported_names_are_neither_qualified_exports_nor_transitive_exports() {
    for source in [
        b"main(){return middle::work();}".as_slice(),
        b"#include middle;main(){return work();}",
        b"main(){return middle::work;}",
    ] {
        let error = compile(
            &[
                (b"root", source),
                (b"middle", b"#include lib;own(){}"),
                (b"lib", b"work(){return 1;}"),
            ],
            Disabled,
            ScriptPackageBudget::default(),
        )
        .expect_err("No imported re-export");
        assert_eq!(error.module_index(), Some(0));
        assert_eq!(error.cause.kind, FunctionCompileErrorKind::UnknownFunction);
    }
}
