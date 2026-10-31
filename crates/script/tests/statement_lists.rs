// SPDX-License-Identifier: AGPL-3.0-or-later
//! Body indices retain complete immediate source order without flattening nested bodies.
#![allow(
    clippy::expect_used,
    reason = "Tests use finite valid synthetic body lists."
)]

use cod4x_script::statement::{
    LegacyStatementKind, LegacyStatementListSyntax, LegacyStatementSyntax, StatementListBudget,
    StatementListErrorKind,
};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNodeKind};

use crate::expression_support;
use crate::statement_support;

#[test]
fn source_order_reverse_order_nested_body_and_list_prefixes_keep_original_nodes()
-> Result<(), String> {
    let tree = expression_support::parse(b"main(){x=1;{y=2;}return x;}", LegacySyntaxMode::Script);
    let body = statement_support::body(&tree);
    let list =
        LegacyStatementListSyntax::lower(body, StatementListBudget::default()).expect("Body");
    let expected = [b"x=1;".as_slice(), b"{y=2;}", b"return x;"];
    assert_eq!(
        list.statements()
            .map(|node| expression_support::bytes(&tree, node))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        list.statements()
            .rev()
            .map(|node| expression_support::bytes(&tree, node))
            .collect::<Vec<_>>(),
        expected.into_iter().rev().collect::<Vec<_>>()
    );
    assert_eq!(list.usage().statements, 3);
    assert_eq!(list.usage().steps, 8);
    assert_eq!(list.syntax().kind(), body.kind());
    assert_eq!(list.span(), body.span());
    let nested = list.statements().nth(1).expect("Nested block");
    let LegacyStatementKind::Block { body: nested_body } = LegacyStatementSyntax::from_node(nested)
        .expect("Block")
        .kind()
    else {
        return Err("Expected block".into());
    };
    let nested = LegacyStatementListSyntax::lower(nested_body, StatementListBudget::default())
        .expect("Separate nested index");
    assert_eq!(nested.usage().statements, 1);
    let prefix = tree.nodes().find(|node| matches!(node.kind(), LegacySyntaxNodeKind::Production(rule) if rule.as_u16() == 109)).expect("First body prefix");
    let prefix =
        LegacyStatementListSyntax::lower(prefix, StatementListBudget::default()).expect("Prefix");
    assert_eq!(prefix.usage().statements, 1);
    assert_eq!(
        expression_support::bytes(&tree, prefix.statements().next().expect("First")),
        b"x=1;"
    );
    Ok(())
}

#[test]
fn seventy_thousand_immediate_statements_are_retained_and_dropped_without_native_count_caps() {
    let text = format!("main(){{{}}}", ";".repeat(70_000));
    let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Script);
    let list = LegacyStatementListSyntax::lower(
        statement_support::body(&tree),
        StatementListBudget::default(),
    )
    .expect("Wide body index");
    assert_eq!(list.usage().statements, 70_000);
    assert_eq!(list.statements().len(), 70_000);
    assert_eq!(list.usage().steps, 140_002);
    assert!(list.statements().all(|node| {
        matches!(
            LegacyStatementSyntax::from_node(node)
                .expect("Empty statement")
                .kind(),
            LegacyStatementKind::Empty
        )
    }));
}

#[test]
fn independent_private_body_indices_and_category_errors_contain_no_source() {
    let first = expression_support::parse(b"main(){PrivateOne=1;}", LegacySyntaxMode::Script);
    let second = expression_support::parse(b"main(){PrivateTwo=2;}", LegacySyntaxMode::Script);
    for (tree, expected) in [
        (&first, b"PrivateOne=1;".as_slice()),
        (&second, b"PrivateTwo=2;".as_slice()),
    ] {
        let body = statement_support::body(tree);
        let list = LegacyStatementListSyntax::lower(body, StatementListBudget::default())
            .expect("Independent body");
        let statement = list.statements().next().expect("Statement");
        assert_eq!(expression_support::bytes(tree, statement), expected);
        assert!(!format!("{list:?}").contains("Private"));
        let error = LegacyStatementListSyntax::lower(statement, StatementListBudget::default())
            .expect_err("Statement is not list");
        assert_eq!(error.kind, StatementListErrorKind::NotStatementList);
        assert_eq!(error.offset, statement.span().start);
        assert_eq!(error.work_steps, 0);
        assert_eq!(error.admitted_statements, 0);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}
