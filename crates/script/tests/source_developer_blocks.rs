// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Developer lists must validate separately from runtime authority and retained table storage."
)]
use crate::{
    array_read_support::number, module_compile_support, source_developer_support::compile,
};
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::KernelCallBudget;

#[test]
fn enabled_assignments_run_and_disabled_or_empty_regions_leave_retained_execution_unchanged() {
    for source in [
        b"main(){n=1;/# n+=41; #/return n;}".as_slice(),
        b"main(){n=1;/# #/ /# n+=20; #/ /# n+=21; #/return n;}",
    ] {
        for (mode, expected) in [(Enabled, 42), (Disabled, 1)] {
            let module = compile(source, mode);
            assert_eq!(
                module
                    .program()
                    .execute(0, &[], KernelCallBudget::default())
                    .expect("Selected region")
                    .value,
                number(expected)
            );
        }
    }
}

#[test]
fn discarded_unknown_names_references_arrays_literals_fields_and_labels_leave_no_executable_rows() {
    let source = br#"main(){/# x=PrivateMissing("PrivateValue");f=::PrivateMissing;f=unknown_file::missing();
        a=[];a[0]="PrivateArray";o=spawnstruct();o.PrivateField="PrivateText";
        switch("PrivateKey"){case "PrivateKey":x=missing();break;} #/return 42;}"#;
    let module = compile(source, Disabled);
    let usage = module.usage();
    assert_eq!(
        (
            usage.string_literals,
            usage.field_names,
            usage.switch_labels
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (
            usage.staged_string_bytes,
            usage.staged_field_bytes,
            usage.staged_switch_bytes
        ),
        (0, 0, 0)
    );
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("No fabricated heap or target")
            .value,
        number(42)
    );
    let active =
        module_compile_support::compile(source, Enabled, ModuleCompilationBudget::default())
            .expect_err("Active unresolved name");
    assert_eq!(active.cause.kind, FunctionCompileErrorKind::UnknownFunction);
    assert!(!format!("{active:?} {active}").contains("Private"));
}

#[test]
fn discarded_arguments_and_private_cells_still_undergo_source_validation() {
    for source in [
        b"main(){/# missing(PrivateUninitialized); #/return 0;}".as_slice(),
        b"main(){/# x=1; #/return x;}",
        b"main(){/# a[0]=1; #/return 0;}",
    ] {
        let error =
            module_compile_support::compile(source, Disabled, ModuleCompilationBudget::default())
                .expect_err("Discard does not invent initialization");
        assert_eq!(
            error.cause.kind,
            FunctionCompileErrorKind::UninitializedLocal
        );
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}

#[test]
fn enabled_returns_leave_parent_validation_open_while_disabled_returns_do_not_execute() {
    let source = b"main(){/# return 42; #/return 9;}";
    for (mode, expected) in [(Enabled, 42), (Disabled, 9)] {
        assert_eq!(
            compile(source, mode)
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect("Conditional developer return")
                .value,
            number(expected)
        );
    }
}
