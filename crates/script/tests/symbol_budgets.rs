// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit index budgets, full declaration retention, and wide function/name counts.
#![allow(
    clippy::expect_used,
    reason = "Tests construct exact synthetic symbol budgets."
)]

use std::fmt::Write;

use cod4x_script::binding::{
    LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget, SymbolErrorKind, SymbolResource,
};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

#[test]
fn every_index_budget_accepts_exact_usage_and_rejects_one_less() {
    let source = SourceText::index(
        b"Main(a) {} /# Hidden() {} #/ Last() {}",
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
    for mode in [LegacyDeveloperMode::Disabled, LegacyDeveloperMode::Enabled] {
        let usage = LegacyModuleSymbols::bind(&module, mode, SymbolBudget::default())
            .expect("Index")
            .usage();
        let exact = SymbolBudget {
            max_functions: usage.functions,
            max_name_bytes: usage.name_bytes,
            max_single_name_bytes: usage.largest_name_bytes,
            max_steps: usage.steps,
            max_metadata_bytes: usage.metadata_bytes,
        };
        assert!(LegacyModuleSymbols::bind(&module, mode, exact).is_ok());
        assert_eq!(usage.largest_name_bytes, 6);
        assert_eq!(
            usage.largest_indexed_name_bytes,
            if mode == LegacyDeveloperMode::Enabled {
                6
            } else {
                4
            }
        );
        for resource in [
            SymbolResource::Functions,
            SymbolResource::NameBytes,
            SymbolResource::SingleNameBytes,
            SymbolResource::Steps,
            SymbolResource::MetadataBytes,
        ] {
            let mut budget = exact;
            match resource {
                SymbolResource::Functions => budget.max_functions -= 1,
                SymbolResource::NameBytes => budget.max_name_bytes -= 1,
                SymbolResource::SingleNameBytes => budget.max_single_name_bytes -= 1,
                SymbolResource::Steps => budget.max_steps -= 1,
                SymbolResource::MetadataBytes => budget.max_metadata_bytes -= 1,
            }
            let error = LegacyModuleSymbols::bind(&module, mode, budget).expect_err("One less");
            assert_eq!(error.kind, SymbolErrorKind::BudgetExceeded(resource));
            assert!(error.offset <= source.bytes().len() as u64);
        }
    }
}

#[test]
fn empty_module_needs_no_index_storage_or_work() {
    let source = SourceText::index(b"", SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Module");
    let symbols = LegacyModuleSymbols::bind(
        &module,
        LegacyDeveloperMode::Disabled,
        SymbolBudget {
            max_functions: 0,
            max_name_bytes: 0,
            max_single_name_bytes: 0,
            max_steps: 0,
            max_metadata_bytes: 0,
        },
    )
    .expect("Empty index");
    assert!(symbols.functions().is_empty());
    assert_eq!(symbols.usage().metadata_bytes, 0);
    assert_eq!(symbols.usage().steps, 0);
}

#[test]
fn sixty_six_thousand_functions_keep_lookup_and_storage_without_short_ids() {
    let mut bytes = String::new();
    for index in 0..66_000 {
        writeln!(bytes, "f{index}() {{}}").expect("Synthetic source");
    }
    let source = SourceText::index(bytes.as_bytes(), SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Module");
    let symbols = LegacyModuleSymbols::bind(
        &module,
        LegacyDeveloperMode::Disabled,
        SymbolBudget::default(),
    )
    .expect("Wide index");
    assert_eq!(symbols.functions().len(), 66_000);
    assert_eq!(symbols.usage().active_functions, 66_000);
    let last = symbols.functions().last().expect("Last definition");
    assert_eq!(last.name().as_bytes(), b"f65999");
    assert_eq!(
        symbols
            .find(last.name())
            .expect("Lookup")
            .declaration()
            .name()
            .span(),
        last.declaration().name().span()
    );
}

#[test]
fn long_case_distinct_names_preserve_native_identity_without_truncating() {
    for length in [8191, 8192, 100_000] {
        let bytes = format!(
            "{}() {{}}\n{}() {{}}",
            "A".repeat(length),
            "a".repeat(length)
        );
        let source = SourceText::index(bytes.as_bytes(), SourceBudget::default()).expect("Source");
        let tree = LegacySyntaxTree::parse_structure(
            &source,
            LegacySyntaxMode::Script,
            LegacySyntaxBudget::default(),
        )
        .expect("Tree");
        let module =
            LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Module");
        let result = LegacyModuleSymbols::bind(
            &module,
            LegacyDeveloperMode::Enabled,
            SymbolBudget::default(),
        );
        if length < 8192 {
            assert_eq!(
                result.expect_err("Canonical duplicate").kind,
                SymbolErrorKind::DuplicateFunction
            );
        } else {
            let symbols = result.expect("Native exact-case fallback");
            assert_eq!(symbols.usage().functions, 2);
            assert_eq!(symbols.usage().name_bytes, length as u64 * 2);
            assert!(
                symbols
                    .functions()
                    .iter()
                    .all(|function| symbols.find(function.name()).is_some())
            );
        }
    }
}
