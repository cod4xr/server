// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete source-ordered list indices, independent of native push/evaluation order.
#![allow(
    clippy::expect_used,
    reason = "Tests supply bounded synthetic expression-list trees."
)]

use crate::expression_list_support;
use crate::expression_support;

use cod4x_script::expression::{
    ExpressionListBudget, ExpressionListErrorKind, LegacyExpressionListSyntax,
};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNode, LegacySyntaxNodeKind};

#[test]
fn ordered_and_reverse_views_preserve_complete_expressions_without_flattening_descendants()
-> Result<(), String> {
    let text = b"(private_left+1, private_array[2], \"private_secret\", nested(3,4))";
    let tree = expression_support::parse(text, LegacySyntaxMode::Fragment);
    let node = expression_list_support::node(&tree)?;
    let list =
        LegacyExpressionListSyntax::lower(node, ExpressionListBudget::default()).expect("List");
    let expected = [
        b"private_left+1".as_slice(),
        b"private_array[2]",
        b"\"private_secret\"",
        b"nested(3,4)",
    ];
    let actual: Vec<_> = list
        .expressions()
        .map(|node| expression_support::bytes(&tree, node))
        .collect();
    assert_eq!(actual, expected);
    let reverse: Vec<_> = list
        .expressions()
        .rev()
        .map(|node| expression_support::bytes(&tree, node))
        .collect();
    assert_eq!(reverse, expected.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(list.usage().expressions, 4);
    assert_eq!(list.usage().steps, 10);
    assert_eq!(
        list.usage().metadata_bytes,
        4 * size_of::<LegacySyntaxNode<'_, '_>>() as u64
    );
    assert_eq!(list.span(), node.span());
    assert_eq!(list.syntax().span(), node.span());
    assert_eq!(tree.source_bytes(), text);
    assert!(!format!("{list:?}").contains("private"));
    Ok(())
}

#[test]
fn empty_and_single_lists_have_exact_storage_and_work_without_runtime_arity_rules()
-> Result<(), String> {
    let empty_tree = expression_support::parse(b"()", LegacySyntaxMode::Fragment);
    let empty_node = expression_list_support::node(&empty_tree)?;
    let list = LegacyExpressionListSyntax::lower(
        empty_node,
        ExpressionListBudget {
            max_expressions: 0,
            max_steps: 2,
            max_metadata_bytes: 0,
        },
    )
    .expect("Zero storage empty list");
    assert_eq!(list.expressions().len(), 0);
    assert_eq!(list.usage().metadata_bytes, 0);
    assert_eq!(list.usage().steps, 2);
    assert_eq!(list.span().start, list.span().end);
    let single_tree = expression_support::parse(b"(private_only)", LegacySyntaxMode::Fragment);
    let single_node = expression_list_support::node(&single_tree)?;
    let single = LegacyExpressionListSyntax::lower(single_node, ExpressionListBudget::default())
        .expect("Single");
    assert_eq!(single.usage().steps, 4);
    assert_eq!(single.expressions().len(), 1);
    assert_eq!(
        expression_support::bytes(&single_tree, single.expressions().next().expect("Item")),
        b"private_only"
    );
    // Indices into both independently owned trees remain alive together.
    assert_eq!(list.expressions().len(), 0);
    Ok(())
}

#[test]
fn nonempty_prefixes_are_supported_and_other_categories_fail_privately() {
    let tree = expression_support::parse(
        b"(private_a,private_b,private_c)",
        LegacySyntaxMode::Fragment,
    );
    let prefix = tree.nodes().find(|node| matches!(node.kind(), LegacySyntaxNodeKind::Production(rule) if rule.as_u16() == 111)).expect("Prefix");
    let list = LegacyExpressionListSyntax::lower(prefix, ExpressionListBudget::default())
        .expect("Prefix list");
    let actual: Vec<_> = list
        .expressions()
        .map(|node| expression_support::bytes(&tree, node))
        .collect();
    assert_eq!(actual, [b"private_a".as_slice(), b"private_b"]);
    let error = LegacyExpressionListSyntax::lower(tree.root(), ExpressionListBudget::default())
        .expect_err("Not a list");
    assert_eq!(error.kind, ExpressionListErrorKind::ExpectedList);
    assert_eq!(error.offset, tree.root().span().start);
    assert!(!format!("{error:?} {error}").contains("private"));
}

#[test]
fn seventy_thousand_expressions_retain_wide_counts_and_iterative_ownership() -> Result<(), String> {
    let count = 70_000;
    let text = "(".to_owned()
        + &std::iter::repeat_n("1", count)
            .collect::<Vec<_>>()
            .join(",")
        + ")";
    let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
    let node = expression_list_support::node(&tree)?;
    let list = LegacyExpressionListSyntax::lower(node, ExpressionListBudget::default())
        .expect("Large frontend list");
    assert_eq!(list.usage().expressions, count as u64);
    assert_eq!(list.usage().steps, 2 * (count as u64 + 1));
    assert_eq!(list.expressions().len(), count);
    assert_eq!(list.expressions().rev().len(), count);
    assert_eq!(
        expression_support::bytes(&tree, list.expressions().nth(65_536).expect("Wide index")),
        b"1"
    );
    assert_eq!(tree.source_bytes(), text.as_bytes());
    Ok(())
}
