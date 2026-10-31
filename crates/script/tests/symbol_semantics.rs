// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local definitions and developer-region rules, not original compiler execution.
#![allow(
    clippy::expect_used,
    reason = "Tests build exact synthetic binding scenarios."
)]

use cod4x_script::binding::{
    LegacyDeveloperMode, LegacyFunctionInclusion, LegacyFunctionVisibility, LegacyIdentifier,
    LegacyModuleSymbols, SymbolBudget, SymbolErrorKind,
};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::lexical::{LegacyLexBudget, LegacyLexer};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

#[test]
fn developer_mode_controls_active_lookup_while_preserving_every_definition() {
    let bytes = b"Private_Main(A,a) {} /# Private_Hidden() {} #/ Private_Tail() {}";
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Module");
    for mode in [LegacyDeveloperMode::Disabled, LegacyDeveloperMode::Enabled] {
        let symbols =
            LegacyModuleSymbols::bind(&module, mode, SymbolBudget::default()).expect("Bindings");
        assert_eq!(symbols.mode(), mode);
        assert_eq!(symbols.functions().len(), 3);
        assert_eq!(symbols.usage().ordinary_functions, 2);
        assert_eq!(symbols.usage().developer_functions, 1);
        assert_eq!(
            symbols.usage().active_functions,
            if mode == LegacyDeveloperMode::Enabled {
                3
            } else {
                2
            }
        );
        let query =
            SourceText::index(b"pRiVaTe_MaIn", SourceBudget::default()).expect("Independent query");
        let name = LegacyIdentifier::from_lexeme(
            LegacyLexer::new(&query, LegacyLexBudget::default())
                .next()
                .expect("Name")
                .expect("Lexeme"),
        )
        .expect("Identifier");
        let found = symbols
            .find(name)
            .expect("Canonical lookup from another source");
        assert_eq!(found.declaration().name().as_bytes(), b"Private_Main");
        assert_eq!(found.visibility(), LegacyFunctionVisibility::Ordinary);
        let hidden = symbols.functions()[1];
        assert_eq!(hidden.visibility(), LegacyFunctionVisibility::DeveloperOnly);
        assert_eq!(
            hidden.inclusion(),
            if mode == LegacyDeveloperMode::Enabled {
                LegacyFunctionInclusion::Active
            } else {
                LegacyFunctionInclusion::DeveloperDisabled
            }
        );
        assert_eq!(
            symbols.find(hidden.name()).is_some(),
            mode == LegacyDeveloperMode::Enabled
        );
        assert!(!format!("{symbols:?} {:?}", symbols.functions()).contains("Private"));
    }
}

#[test]
fn duplicate_checks_use_canonical_active_names_and_leave_declarations_unchanged() {
    for bytes in [
        b"Private_Secret() {} private_secret() {}".as_slice(),
        b"Private_Secret() {} /# private_secret() {} #/",
        b"/# Private_Secret() {} private_secret() {} #/",
    ] {
        let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
        let tree = LegacySyntaxTree::parse_structure(
            &source,
            LegacySyntaxMode::Script,
            LegacySyntaxBudget::default(),
        )
        .expect("Tree");
        let module =
            LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Module");
        let usage = module.usage();
        let error = LegacyModuleSymbols::bind(
            &module,
            LegacyDeveloperMode::Enabled,
            SymbolBudget::default(),
        )
        .expect_err("Duplicate");
        let text = std::str::from_utf8(bytes).expect("ASCII");
        assert_eq!(error.kind, SymbolErrorKind::DuplicateFunction);
        assert_eq!(
            error.offset,
            text.find("private_secret").expect("Second declaration") as u64
        );
        assert_eq!(
            error.previous_offset,
            Some(text.find("Private_Secret").expect("First declaration") as u64)
        );
        assert!(
            !format!("{error:?} {error}")
                .to_lowercase()
                .contains("private_secret")
        );
        assert_eq!(module.usage(), usage);
        let disabled = LegacyModuleSymbols::bind(
            &module,
            LegacyDeveloperMode::Disabled,
            SymbolBudget::default(),
        );
        if text.contains("/#") {
            assert!(disabled.is_ok());
        } else {
            assert_eq!(
                disabled.expect_err("Ordinary duplicate").kind,
                SymbolErrorKind::DuplicateFunction
            );
        }
    }
}

#[test]
fn region_errors_and_directive_phase_order_have_original_source_offsets() {
    for (bytes, kind, offset, previous) in [
        (
            b"/# /# private() {} #/ #/".as_slice(),
            SymbolErrorKind::NestedDeveloperRegion,
            3,
            Some(0),
        ),
        (
            b"#/ private() {}".as_slice(),
            SymbolErrorKind::UnexpectedDeveloperEnd,
            0,
            None,
        ),
        (
            b"/# private() {}".as_slice(),
            SymbolErrorKind::UnclosedDeveloperRegion,
            0,
            None,
        ),
        (
            b"/# #using_animtree(\"private-tree\"); #/".as_slice(),
            SymbolErrorKind::AnimTreeInDeveloperRegion,
            3,
            Some(0),
        ),
        (
            b"/# #using_animtree(\"private-tree\");".as_slice(),
            SymbolErrorKind::UnclosedDeveloperRegion,
            0,
            None,
        ),
    ] {
        let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
        let tree = LegacySyntaxTree::parse_structure(
            &source,
            LegacySyntaxMode::Script,
            LegacySyntaxBudget::default(),
        )
        .expect("Structure");
        let module =
            LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Declarations");
        for mode in [LegacyDeveloperMode::Disabled, LegacyDeveloperMode::Enabled] {
            let error = LegacyModuleSymbols::bind(&module, mode, SymbolBudget::default())
                .expect_err("Invalid region");
            assert_eq!(error.kind, kind);
            assert_eq!(error.offset, offset);
            assert_eq!(error.previous_offset, previous);
            assert!(!format!("{error:?} {error}").contains("private"));
        }
    }
    let source = SourceText::index(
        b"/# #using_animtree(\"x\"); #/ Main() {} main() {}",
        SourceBudget::default(),
    )
    .expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Module");
    assert_eq!(
        LegacyModuleSymbols::bind(
            &module,
            LegacyDeveloperMode::Enabled,
            SymbolBudget::default()
        )
        .expect_err("Definitions precede directive placement")
        .kind,
        SymbolErrorKind::DuplicateFunction
    );
}
