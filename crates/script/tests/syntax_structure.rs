// SPDX-License-Identifier: AGPL-3.0-or-later
//! Iterative ownership, original spans, and separation from literal semantics.
#![allow(
    clippy::expect_used,
    reason = "Tests construct exact synthetic syntax trees."
)]

use cod4x_script::literal::{LegacyLiteral, LegacyLiteralBudget};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{
    LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxNode, LegacySyntaxNodeKind, LegacySyntaxTree,
};

#[test]
fn borrowed_terminals_and_ordered_children_retain_exact_source_locations() -> Result<(), String> {
    let bytes = b"/*private-comment*/\nmain() { return \"private-string\"; }\n";
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    assert_eq!(tree.source_bytes().as_ptr(), bytes.as_ptr());
    assert_eq!(tree.mode(), LegacySyntaxMode::Script);
    let mut source_tokens = 0;
    let mut edges = 0;
    let mut strings = 0;
    for node in tree.nodes() {
        let span = node.span();
        assert!(span.start <= span.end && span.end <= bytes.len() as u64);
        assert!(!format!("{node:?}").contains("private"));
        edges += node.children().len() as u64;
        for child in node.children() {
            let child_span = child.span();
            assert!(span.start <= child_span.start && child_span.end <= span.end);
        }
        if let Some(lexeme) = node.lexeme() {
            source_tokens += 1;
            assert_eq!(lexeme.span(), span);
            assert_eq!(lexeme.as_bytes(), source.span(span.clone()).expect("Span"));
            assert_eq!(
                lexeme.as_bytes().as_ptr(),
                bytes[usize::try_from(span.start).expect("Offset")..].as_ptr()
            );
            if lexeme.as_bytes() == b"\"private-string\"" {
                let literal = LegacyLiteral::decode(lexeme, LegacyLiteralBudget::default())
                    .expect("Literal stage");
                let LegacyLiteral::String(value) = literal else {
                    return Err("Expected string literal".to_owned());
                };
                assert_eq!(value.as_bytes(), b"private-string");
                strings += 1;
            }
        }
    }
    assert_eq!(strings, 1);
    assert_eq!(source_tokens, tree.usage().source_tokens);
    assert_eq!(edges, tree.usage().child_edges);
    assert!(!format!("{tree:?}").contains("private"));
    Ok(())
}

#[test]
fn twelve_thousand_nested_parentheses_use_growable_stack_and_flat_destruction() {
    let depth = 12_000;
    let bytes = format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
    let source = SourceText::index(bytes.as_bytes(), SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Fragment,
        LegacySyntaxBudget::default(),
    )
    .expect("Deep tree");
    assert!(tree.usage().peak_stack_entries > 10_000);
    assert_eq!(tree.usage().source_tokens, depth as u64 * 2 + 1);
    assert!(tree.usage().nodes > 24_000);
    drop(tree);
}

#[test]
fn structural_success_does_not_imply_literal_range_or_name_binding() {
    let bytes = b"main() { return unknown_name + 999999999999999999999999999999; }";
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Structure only");
    let literal = tree
        .nodes()
        .filter_map(LegacySyntaxNode::lexeme)
        .find(|lexeme| lexeme.as_bytes().starts_with(b"999"))
        .expect("Integer token");
    assert!(LegacyLiteral::decode(literal, LegacyLiteralBudget::default()).is_err());
    assert!(matches!(
        tree.root().kind(),
        LegacySyntaxNodeKind::Production(_)
    ));
}
