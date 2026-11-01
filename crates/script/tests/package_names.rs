// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests admit complete virtual names and then destroy caller-owned input bytes."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::{FunctionCompileErrorKind, ScriptPackageBudget};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn duplicate_empty_nul_and_non_ascii_module_names_are_rejected_privately() {
    let error = compile(
        &[(b"//Private\\Library", b""), (b"private/library", b"")],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect_err("Canonical collision");
    assert_eq!(error.module_index(), Some(1));
    assert_eq!(error.cause.kind, FunctionCompileErrorKind::DuplicateModule);
    assert!(!format!("{error:?} {error}").contains("Private"));
    for name in [b"".as_slice(), b"\n\r\t"] {
        let error = compile(&[(name, b"")], Disabled, ScriptPackageBudget::default())
            .expect_err("No empty package identity");
        assert_eq!(error.cause.kind, FunctionCompileErrorKind::EmptyModuleName);
    }
    for name in [b"Private\0Other".as_slice(), b"Private\xff"] {
        let error = compile(&[(name, b"")], Disabled, ScriptPackageBudget::default())
            .expect_err("Invalid virtual identity");
        assert!(matches!(
            error.cause.kind,
            FunctionCompileErrorKind::ScriptPath(_)
        ));
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn owned_package_keeps_complete_wide_names_code_and_maps_after_input_drop() {
    let mut name = vec![b'A'; 70_000];
    let mut root = b"main(){return ".to_vec();
    root.extend_from_slice(&name);
    root.extend_from_slice(b"::helper();}");
    let mut library = b"helper(){return 23;}".to_vec();
    let package = compile(
        &[(b"root", &root), (&name, &library)],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("No native filename ceiling");
    name.fill(0);
    root.fill(0);
    library.fill(0);
    drop((name, root, library));
    assert_eq!(
        package.module_name(1).expect("Owned name"),
        vec![b'a'; 70_000]
    );
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Owned code")
            .value,
        KernelValue::Number(LegacyNumber::Integer(23))
    );
    assert_eq!(
        package
            .instruction_location(1, 0)
            .expect("Owned map")
            .module,
        1
    );
}
