// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded wrappers, independent trees, category failures, privacy, and deep local views.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid synthetic immutable statement trees."
)]

use cod4x_script::statement::{LegacyStatementError, LegacyStatementKind, LegacyStatementSyntax};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNodeKind};

use crate::expression_support;
use crate::statement_support;

#[test]
fn direct_and_wrapped_views_preserve_supplied_nodes_and_private_independent_sources()
-> Result<(), String> {
    let first = expression_support::parse(b"main(){PrivateFirst=1;}", LegacySyntaxMode::Script);
    let second = expression_support::parse(b"main(){PrivateSecond=2;}", LegacySyntaxMode::Script);
    for (tree, expected) in [
        (&first, b"PrivateFirst".as_slice()),
        (&second, b"PrivateSecond".as_slice()),
    ] {
        let mut count = 0;
        for node in tree.nodes().filter(|node| matches!(node.kind(), LegacySyntaxNodeKind::Production(rule) if matches!(rule.as_u16(), 67 | 94 | 97 | 108))) {
            let view = LegacyStatementSyntax::from_node(node).expect("Direct or wrapped assignment");
            assert_eq!(view.syntax().kind(), node.kind());
            assert!(matches!(view.statement_syntax().kind(), LegacySyntaxNodeKind::Production(rule) if rule.as_u16() == 67));
            assert_eq!(view.span(), node.span());
            let LegacyStatementKind::Assignment { target, .. } = view.kind() else { return Err("Expected assignment".into()); };
            assert_eq!(expression_support::bytes(tree, target), expected);
            let debug = format!("{view:?}");
            assert!(!debug.contains("PrivateFirst") && !debug.contains("PrivateSecond"));
            count += 1;
        }
        assert_eq!(count, 4);
        assert_eq!(
            LegacyStatementSyntax::from_node(statement_support::body(tree))
                .expect_err("List is not statement"),
            LegacyStatementError::NotStatement
        );
    }
    let tree = expression_support::parse(b"1+2", LegacySyntaxMode::Fragment);
    for node in tree.nodes() {
        assert_eq!(
            LegacyStatementSyntax::from_node(node).expect_err("Expression is not statement"),
            LegacyStatementError::NotStatement
        );
    }
    Ok(())
}

#[test]
fn twelve_thousand_nested_conditionals_are_viewed_and_dropped_iteratively() -> Result<(), String> {
    let text = format!("main(){{{}return 1;}}", "if(true)".repeat(12_000));
    let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Script);
    let mut view = statement_support::first(&tree);
    for _ in 0..12_000 {
        let LegacyStatementKind::If {
            consequence,
            alternative,
            ..
        } = view.kind()
        else {
            return Err("Expected nested if".into());
        };
        assert!(alternative.is_none());
        view = LegacyStatementSyntax::from_node(consequence).expect("Next local statement");
    }
    assert!(matches!(
        view.kind(),
        LegacyStatementKind::Return { value: Some(_) }
    ));
    Ok(())
}
