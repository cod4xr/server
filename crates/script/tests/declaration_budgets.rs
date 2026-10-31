// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded metadata/work and declaration counts beyond old packed capacities.
#![allow(
    clippy::expect_used,
    reason = "Tests construct synthetic modules and exact budgets."
)]

use std::fmt::Write;

use cod4x_script::declaration::{
    DeclarationBudget, DeclarationErrorKind, DeclarationResource, LegacyModuleItem,
    LegacyModuleSyntax,
};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

#[test]
fn all_budgets_accept_exact_usage_and_reject_one_less_without_mutating_tree() {
    let source = SourceText::index(
        b"#include common; Main(a,b) {} Other(c) {}",
        SourceBudget::default(),
    )
    .expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let original = tree.usage();
    let usage = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default())
        .expect("Module")
        .usage();
    let exact = DeclarationBudget {
        max_items: usage.items,
        max_includes: usage.includes,
        max_parameters: usage.parameters,
        max_steps: usage.steps,
        max_metadata_bytes: usage.metadata_bytes,
    };
    assert!(LegacyModuleSyntax::lower(&tree, exact).is_ok());
    for resource in [
        DeclarationResource::Items,
        DeclarationResource::Includes,
        DeclarationResource::Parameters,
        DeclarationResource::Steps,
        DeclarationResource::MetadataBytes,
    ] {
        let mut budget = exact;
        match resource {
            DeclarationResource::Items => budget.max_items -= 1,
            DeclarationResource::Includes => budget.max_includes -= 1,
            DeclarationResource::Parameters => budget.max_parameters -= 1,
            DeclarationResource::Steps => budget.max_steps -= 1,
            DeclarationResource::MetadataBytes => budget.max_metadata_bytes -= 1,
        }
        let error = LegacyModuleSyntax::lower(&tree, budget).expect_err("One less");
        assert_eq!(error.kind, DeclarationErrorKind::BudgetExceeded(resource));
        assert!(error.offset <= source.bytes().len() as u64);
        assert_eq!(tree.usage(), original);
    }
}

#[test]
fn empty_module_needs_no_declaration_storage_and_errors_do_not_expose_names() {
    let source = SourceText::index(b"", SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module = LegacyModuleSyntax::lower(
        &tree,
        DeclarationBudget {
            max_items: 0,
            max_includes: 0,
            max_parameters: 0,
            max_steps: tree.usage().nodes,
            max_metadata_bytes: 0,
        },
    )
    .expect("Empty module");
    assert_eq!(module.usage().metadata_bytes, 0);
    let source =
        SourceText::index(b"private_secret() {}", SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let error = LegacyModuleSyntax::lower(
        &tree,
        DeclarationBudget {
            max_items: 0,
            ..DeclarationBudget::default()
        },
    )
    .expect_err("Budget");
    assert!(!format!("{error:?} {error}").contains("private"));
}

#[test]
fn wide_module_counts_and_parameter_lists_do_not_truncate_to_legacy_id_widths() -> Result<(), String>
{
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
    .expect("Wide tree");
    let module =
        LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Wide declarations");
    assert_eq!(module.items().len(), 66_000);
    let parameters = (0..1024)
        .map(|index| format!("p{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let bytes = format!("Wide({parameters}) {{}}");
    let source = SourceText::index(bytes.as_bytes(), SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module =
        LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Parameters");
    let Some(LegacyModuleItem::Function(function)) = module.items().first() else {
        return Err("Expected function declaration".to_owned());
    };
    assert_eq!(function.parameters().len(), 1024);
    assert_eq!(function.parameters()[0].as_bytes(), b"p0");
    assert_eq!(function.parameters()[1023].as_bytes(), b"p1023");
    assert_eq!(module.usage().parameters, 1024);
    Ok(())
}
