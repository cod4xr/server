// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed source operators preserve operands and do not bind/evaluate descendants.
#![allow(
    clippy::expect_used,
    reason = "Tests supply synthetic structurally valid expressions."
)]

mod expression_support;

use cod4x_script::expression::{
    LegacyExpressionKind, LegacyExpressionSyntax, LegacyInfixOperator, LegacyLogicalOperator,
    LegacyPrefixOperator,
};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNodeKind};

#[test]
fn all_eager_infix_operators_retain_original_order_spans_and_private_diagnostics()
-> Result<(), String> {
    for (spelling, expected) in [
        ("|", LegacyInfixOperator::BitwiseOr),
        ("^", LegacyInfixOperator::BitwiseExclusiveOr),
        ("&", LegacyInfixOperator::BitwiseAnd),
        ("==", LegacyInfixOperator::Equal),
        ("!=", LegacyInfixOperator::NotEqual),
        ("<", LegacyInfixOperator::Less),
        (">", LegacyInfixOperator::Greater),
        ("<=", LegacyInfixOperator::LessOrEqual),
        (">=", LegacyInfixOperator::GreaterOrEqual),
        ("<<", LegacyInfixOperator::ShiftLeft),
        (">>", LegacyInfixOperator::ShiftRight),
        ("+", LegacyInfixOperator::Add),
        ("-", LegacyInfixOperator::Subtract),
        ("*", LegacyInfixOperator::Multiply),
        ("/", LegacyInfixOperator::Divide),
        ("%", LegacyInfixOperator::Remainder),
    ] {
        let text = format!("private_left /* comment */ {spelling} private_right");
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyExpressionSyntax::from_node(tree.root()).expect("Expression");
        let LegacyExpressionKind::Infix {
            operator,
            left,
            right,
        } = view.kind()
        else {
            return Err("Expected eager infix syntax".to_owned());
        };
        assert_eq!(operator, expected);
        assert_eq!(expression_support::bytes(&tree, left), b"private_left");
        assert_eq!(expression_support::bytes(&tree, right), b"private_right");
        assert_eq!(view.span(), 0..text.len() as u64);
        assert_eq!(view.syntax().span(), tree.root().span());
        assert!(!format!("{view:?}").contains("private"));
    }
    Ok(())
}

#[test]
fn logical_operators_keep_the_right_branch_unevaluated_and_separate_from_eager_operators()
-> Result<(), String> {
    for (text, expected, left_bytes) in [
        (
            "false && missing()",
            LegacyLogicalOperator::And,
            b"false".as_slice(),
        ),
        (
            "true || missing()",
            LegacyLogicalOperator::Or,
            b"true".as_slice(),
        ),
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyExpressionSyntax::from_node(tree.root()).expect("Expression");
        let LegacyExpressionKind::Logical {
            operator,
            left,
            right,
        } = view.kind()
        else {
            return Err("Expected short-circuit syntax".to_owned());
        };
        assert_eq!(operator, expected);
        assert_eq!(expression_support::bytes(&tree, left), left_bytes);
        assert_eq!(expression_support::bytes(&tree, right), b"missing()");
        let call = LegacyExpressionSyntax::from_node(right).expect("Opaque call");
        assert!(matches!(call.kind(), LegacyExpressionKind::Primary { .. }));
    }
    Ok(())
}

#[test]
fn prefix_operators_do_not_reinterpret_negative_literal_productions() -> Result<(), String> {
    for (text, expected) in [
        ("!private_operand", LegacyPrefixOperator::LogicalNot),
        ("~private_operand", LegacyPrefixOperator::BitwiseNot),
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyExpressionSyntax::from_node(tree.root()).expect("Expression");
        let LegacyExpressionKind::Prefix { operator, operand } = view.kind() else {
            return Err("Expected prefix syntax".to_owned());
        };
        assert_eq!(operator, expected);
        assert_eq!(
            expression_support::bytes(&tree, operand),
            b"private_operand"
        );
    }
    let tree = expression_support::parse(b"-3", LegacySyntaxMode::Fragment);
    let view = LegacyExpressionSyntax::from_node(tree.root()).expect("Expression");
    let LegacyExpressionKind::Primary { syntax } = view.kind() else {
        return Err("Expected opaque signed literal".to_owned());
    };
    let LegacySyntaxNodeKind::Production(rule) = syntax.kind() else {
        return Err("Expected primitive production".to_owned());
    };
    assert_eq!(rule.as_u16(), 43);
    assert_eq!(expression_support::bytes(&tree, syntax), b"-3");
    Ok(())
}
