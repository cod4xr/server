// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute qualified/local calls in an owned closed package after source metadata has dropped."
)]
use crate::package_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ScriptPackageBudget;
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn explicit_file_calls_resolve_canonical_names_and_keep_unqualified_names_local() {
    let package = package_compile_support::compile(
        &[
            (
                b"ROOT",
                b"main(){return maps\\lib::main()+helper();}helper(){return 4;}",
            ),
            (
                b"//MAPS//LIB",
                b"main(){return helper();}helper(){return 8;}",
            ),
        ],
        LegacyDeveloperMode::Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Closed package");
    let entry = package.function_for_declaration(0, 0).expect("Root entry");
    assert_eq!(
        package
            .program()
            .execute(entry, &[], KernelCallBudget::default())
            .expect("Cross-file execution")
            .value,
        KernelValue::Number(LegacyNumber::Integer(12))
    );
    assert_eq!(package.module_name(0), Some(b"root".as_slice()));
    assert_eq!(package.module_name(1), Some(b"maps/lib".as_slice()));
    assert_eq!(package.modules()[0].functions, 0..2);
    assert_eq!(package.modules()[1].functions, 2..4);
    assert_eq!(package.function_for_declaration(1, 1), Some(3));
    let location = package.instruction_location(2, 0).expect("Library map");
    assert_eq!(location.module, 1);
    assert!(location.span.end > location.span.start);
    assert_eq!(package.function_for_declaration(2, 0), None);
    assert_eq!(package.function_for_declaration(0, 2), None);
    assert_eq!(package.instruction_location(u64::MAX, 0), None);
}

#[test]
fn forward_qualified_function_references_and_mutually_recursive_files_share_one_image() {
    let package = package_compile_support::compile(&[
        (b"a", b"main(){f=b::twice;return [[f]](a::even(20));}even(n){if(n==0){return 1;}else{return b::odd(n-1);}}"),
        (b"b", b"twice(n){return n*2;}odd(n){if(n==0){return 0;}else{return a::even(n-1);}}"),
    ], LegacyDeveloperMode::Disabled, ScriptPackageBudget::default()).expect("Forward/cyclic namespace inventory");
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Indirect and cross-file recursive execution")
            .value,
        KernelValue::Number(LegacyNumber::Integer(2))
    );
    assert_eq!(package.usage().modules, 2);
    assert_eq!(package.usage().compilation.functions, 4);
    assert_eq!(package.usage().canonical_module_bytes, 2);
}
