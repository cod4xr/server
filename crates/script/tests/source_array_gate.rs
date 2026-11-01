// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests keep unfinished array lifetime/COW behavior out of executable modules."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};

#[test]
fn empty_array_source_remains_rejected_until_complete_vm_ownership_and_cow_are_ready() {
    for source in [
        b"main(){privateArray=[];return privateArray;}".as_slice(),
        b"main(){return [];}",
    ] {
        let error = module_compile_support::compile(
            source,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect_err("No source activation from empty-data component work");
        assert_eq!(
            error.cause.kind,
            FunctionCompileErrorKind::UnsupportedPrimary
        );
        assert!(!format!("{error:?} {error}").contains("privateArray"));
    }
}
