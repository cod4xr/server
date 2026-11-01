// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests keep unfinished array lifetime/COW behavior out of executable modules."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{
    LegacySyntaxBudget, LegacySyntaxErrorKind, LegacySyntaxMode, LegacySyntaxTree,
};

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
    for source in [
        "main(a){a[0]=1;return 0;}",
        "main(a){a[0]+=1;return 0;}",
        "main(a){a[0]++;return 0;}",
        "main(){game[0]=1;return 0;}",
    ] {
        let error = module_compile_support::compile(
            source.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect_err("Read-only access must not enable indexed writes");
        assert_eq!(
            error.cause.kind,
            FunctionCompileErrorKind::UnsupportedLocalTarget
        );
    }
    let source =
        SourceText::index(b"main(a){game=a;return 0;}", SourceBudget::default()).expect("Source");
    let error = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect_err("Game replacement remains outside the grammar");
    assert_eq!(error.kind, LegacySyntaxErrorKind::UnexpectedToken);
}
