// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed top-level views retain source ownership, order, and unresolved semantics.
#![allow(
    clippy::expect_used,
    reason = "Tests build exact synthetic declaration trees."
)]

use cod4x_script::declaration::{
    DeclarationBudget, DeclarationErrorKind, LegacyModuleItem, LegacyModuleSyntax,
};
use cod4x_script::literal::{LegacyLiteral, LegacyLiteralBudget};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

#[test]
fn includes_functions_parameters_and_developer_markers_preserve_source_order_and_spans()
-> Result<(), String> {
    let bytes = b"#include maps\\mp\\Private; #include Common; #include Common;\n/# Hidden(A,a) { return A; } #/\n#using_animtree(\"private-tree\"); Main() {}";
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module =
        LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Declarations");
    let paths: Vec<_> = module
        .includes()
        .iter()
        .map(|lexeme| lexeme.as_bytes())
        .collect();
    assert_eq!(
        paths,
        [b"maps\\mp\\Private".as_slice(), b"Common", b"Common"]
    );
    assert_eq!(module.items().len(), 5);
    assert!(matches!(
        module.items()[0],
        LegacyModuleItem::DeveloperBegin { .. }
    ));
    let LegacyModuleItem::Function(function) = &module.items()[1] else {
        return Err("Expected hidden function".to_owned());
    };
    assert_eq!(function.name().as_bytes(), b"Hidden");
    assert_eq!(
        function
            .parameters()
            .iter()
            .map(|value| value.as_bytes())
            .collect::<Vec<_>>(),
        [b"A", b"a"]
    );
    assert_eq!(
        source.span(function.body().span()).expect("Body"),
        b" return A;"
    );
    assert_eq!(
        source.span(function.span()).expect("Function"),
        b"Hidden(A,a) { return A; }"
    );
    assert_eq!(
        function.name().as_bytes().as_ptr(),
        source.span(function.name().span()).expect("Name").as_ptr()
    );
    assert!(matches!(
        module.items()[2],
        LegacyModuleItem::DeveloperEnd { .. }
    ));
    let LegacyModuleItem::UsingAnimTree(directive) = module.items()[3] else {
        return Err("Expected directive".to_owned());
    };
    let LegacyLiteral::String(name) =
        LegacyLiteral::decode(directive.name(), LegacyLiteralBudget::default())
            .expect("Literal stage")
    else {
        return Err("Expected directive string".to_owned());
    };
    assert_eq!(name.as_bytes(), b"private-tree");
    assert_eq!(
        source.span(directive.span()).expect("Directive span"),
        b"#using_animtree(\"private-tree\");"
    );
    let LegacyModuleItem::Function(main) = &module.items()[4] else {
        return Err("Expected main".to_owned());
    };
    assert_eq!(main.name().as_bytes(), b"Main");
    assert!(main.parameters().is_empty());
    assert_eq!(module.usage().items, 5);
    assert_eq!(module.usage().includes, 3);
    assert_eq!(module.usage().parameters, 2);
    assert!(
        !format!("{module:?} {:?}", module.items())
            .to_lowercase()
            .contains("private")
    );
    assert!(!format!("{module:?} {:?}", module.items()).contains("Hidden"));
    Ok(())
}

#[test]
fn syntax_lowering_retains_duplicate_names_parameters_and_unbalanced_developer_markers() {
    let source = SourceText::index(b"#/ Main(A,A) {} Main() {} /#", SourceBudget::default())
        .expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Structure");
    let module = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default())
        .expect("Unbound declarations");
    assert_eq!(module.items().len(), 4);
    assert_eq!(module.usage().parameters, 2);
    assert!(matches!(
        module.items()[0],
        LegacyModuleItem::DeveloperEnd { offset: 0 }
    ));
    assert!(matches!(
        module.items()[3],
        LegacyModuleItem::DeveloperBegin { .. }
    ));
}

#[test]
fn every_successful_script_replay_lowers_and_fragments_are_explicitly_rejected() {
    for row in include_str!("fixtures/syntax_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        if fields[2] != "ok" {
            continue;
        }
        let bytes: Vec<_> = if fields[1] == "~" {
            Vec::new()
        } else {
            fields[1]
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| {
                    u8::from_str_radix(std::str::from_utf8(pair).expect("Hex"), 16).expect("Byte")
                })
                .collect()
        };
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Source");
        let mode = if fields[0] == "script" {
            LegacySyntaxMode::Script
        } else {
            LegacySyntaxMode::Fragment
        };
        let tree = LegacySyntaxTree::parse_structure(&source, mode, LegacySyntaxBudget::default())
            .expect("Reference structure");
        let result = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default());
        if mode == LegacySyntaxMode::Script {
            assert!(result.is_ok());
        } else {
            assert_eq!(
                result.expect_err("Fragment is not a module").kind,
                DeclarationErrorKind::ExpectedScript
            );
        }
    }
}
