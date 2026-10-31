// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-tree precedence, opaque primaries, category failures, and iterative depth.
#![allow(
    clippy::expect_used,
    reason = "Tests provide synthetic structural source and checked scoped views."
)]

use crate::expression_support;

use cod4x_script::expression::{
    LegacyExpressionError, LegacyExpressionKind, LegacyExpressionSyntax, LegacyInfixOperator,
    LegacyLogicalOperator,
};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNode};

fn infix<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    expected: LegacyInfixOperator,
) -> Result<
    (
        LegacySyntaxNode<'tree, 'source>,
        LegacySyntaxNode<'tree, 'source>,
    ),
    String,
> {
    let view = LegacyExpressionSyntax::from_node(node).map_err(|error| error.to_string())?;
    let LegacyExpressionKind::Infix {
        operator,
        left,
        right,
    } = view.kind()
    else {
        return Err("Expected infix expression".to_owned());
    };
    assert_eq!(operator, expected);
    Ok((left, right))
}

#[test]
fn precedence_and_left_associativity_follow_the_structural_tree_without_reparsing()
-> Result<(), String> {
    let tree = expression_support::parse(b"a+b*c-d/e%f", LegacySyntaxMode::Fragment);
    let (left, right) = infix(tree.root(), LegacyInfixOperator::Subtract)?;
    assert_eq!(expression_support::bytes(&tree, left), b"a+b*c");
    assert_eq!(expression_support::bytes(&tree, right), b"d/e%f");
    let (_, product) = infix(left, LegacyInfixOperator::Add)?;
    let (factor_left, factor_right) = infix(product, LegacyInfixOperator::Multiply)?;
    assert_eq!(expression_support::bytes(&tree, factor_left), b"b");
    assert_eq!(expression_support::bytes(&tree, factor_right), b"c");
    let (quotient, divisor) = infix(right, LegacyInfixOperator::Remainder)?;
    let (numerator, denominator) = infix(quotient, LegacyInfixOperator::Divide)?;
    assert_eq!(expression_support::bytes(&tree, numerator), b"d");
    assert_eq!(expression_support::bytes(&tree, denominator), b"e");
    assert_eq!(expression_support::bytes(&tree, divisor), b"f");
    let other = expression_support::parse(b"a-b-c", LegacySyntaxMode::Fragment);
    let (left, _) = infix(other.root(), LegacyInfixOperator::Subtract)?;
    let (minuend, subtrahend) = infix(left, LegacyInfixOperator::Subtract)?;
    assert_eq!(expression_support::bytes(&other, minuend), b"a");
    assert_eq!(expression_support::bytes(&other, subtrahend), b"b");
    // Both independently owned trees/views remain valid at the same time.
    assert_eq!(expression_support::bytes(&tree, numerator), b"d");
    Ok(())
}

#[test]
fn logical_precedence_retains_nested_short_circuit_branches() -> Result<(), String> {
    let tree = expression_support::parse(b"a||b&&c", LegacySyntaxMode::Fragment);
    let view = LegacyExpressionSyntax::from_node(tree.root()).expect("Root");
    let LegacyExpressionKind::Logical {
        operator,
        left,
        right,
    } = view.kind()
    else {
        return Err("Expected logical OR".to_owned());
    };
    assert_eq!(operator, LegacyLogicalOperator::Or);
    assert_eq!(expression_support::bytes(&tree, left), b"a");
    let view = LegacyExpressionSyntax::from_node(right).expect("Right branch");
    let LegacyExpressionKind::Logical {
        operator,
        left,
        right,
    } = view.kind()
    else {
        return Err("Expected logical AND".to_owned());
    };
    assert_eq!(operator, LegacyLogicalOperator::And);
    assert_eq!(expression_support::bytes(&tree, left), b"b");
    assert_eq!(expression_support::bytes(&tree, right), b"c");
    Ok(())
}

#[test]
fn primary_forms_stay_opaque_without_decoding_literals_binding_calls_or_removing_groups()
-> Result<(), String> {
    for text in [
        "(1)",
        "(1,2,3)",
        "1",
        "1.0",
        "-1",
        "-1.0",
        "\"private_secret\"",
        "&\"private_localized\"",
        "f(1)",
        "self.x",
        "undefined",
        "self",
        "level",
        "game",
        "anim",
        "x.size",
        "::f",
        "[]",
        "%foo",
        "false",
        "true",
        "99999999999999999999999",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyExpressionSyntax::from_node(tree.root()).expect("Expression");
        let LegacyExpressionKind::Primary { syntax } = view.kind() else {
            return Err("Expected opaque primary".to_owned());
        };
        assert_eq!(expression_support::bytes(&tree, syntax), text.as_bytes());
        assert!(!format!("{view:?}").contains("private"));
    }
    Ok(())
}

#[test]
fn nonexpression_nodes_fail_explicitly_without_source_contents() {
    for (text, mode) in [
        ("main(){return 1;}", LegacySyntaxMode::Script),
        ("return 1;", LegacySyntaxMode::Fragment),
        ("{}", LegacySyntaxMode::Fragment),
    ] {
        let tree = expression_support::parse(text.as_bytes(), mode);
        let error = LegacyExpressionSyntax::from_node(tree.root()).expect_err("Wrong category");
        assert_eq!(error, LegacyExpressionError::NotExpression);
        assert!(!format!("{error:?} {error}").contains("main"));
        let terminal = tree
            .nodes()
            .find(|node| node.lexeme().is_some())
            .expect("Terminal");
        assert_eq!(
            LegacyExpressionSyntax::from_node(terminal).expect_err("Terminal"),
            LegacyExpressionError::NotExpression
        );
    }
}

#[test]
fn twelve_thousand_nested_operators_can_be_viewed_iteratively_without_recursive_ownership()
-> Result<(), String> {
    let depth = 12_000;
    let text = "1+".repeat(depth) + "1";
    let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
    let mut node = tree.root();
    for _ in 0..depth {
        let (left, right) = infix(node, LegacyInfixOperator::Add)?;
        assert_eq!(expression_support::bytes(&tree, right), b"1");
        node = left;
    }
    let last = LegacyExpressionSyntax::from_node(node).expect("Final primary");
    assert!(matches!(last.kind(), LegacyExpressionKind::Primary { .. }));
    assert_eq!(expression_support::bytes(&tree, node), b"1");
    Ok(())
}
