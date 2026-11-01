// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests retain the earlier gate identity while guarding excluded indexed targets."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{
    LegacySyntaxBudget, LegacySyntaxErrorKind, LegacySyntaxMode, LegacySyntaxTree,
};

#[test]
// Retain this regression's inventory identity from ADR 0087. ADR 0095 replaces
// its old plain-local case with a nested target; dedicated tests cover activation.
fn indexed_writes_and_game_replacement_remain_rejected_after_empty_array_activation() {
    for source in [
        "main(a){a[0][1]=1;return 0;}",
        "main(o){o.a[0]=1;return 0;}",
        "main(a){a[0]+=1;return 0;}",
        "main(a){a[0]++;return 0;}",
        "main(){game[0]=1;return 0;}",
    ] {
        let error = module_compile_support::compile(
            source.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect_err("Nested, compound, postfix and root indexed targets remain closed");
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
