// SPDX-License-Identifier: AGPL-3.0-or-later
//! Basic statement roles retain original unresolved children and compound identities.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid synthetic source and expected statement shapes."
)]

use cod4x_script::expression::{LegacyCallSyntax, LegacyInfixOperator};
use cod4x_script::statement::{LegacyStatementKind, LegacyUpdateKind};
use cod4x_script::syntax::LegacySyntaxMode;

use crate::expression_support;
use crate::statement_support;

#[test]
fn assignments_updates_returns_waits_and_calls_preserve_original_roles() -> Result<(), String> {
    let mode = LegacySyntaxMode::Script;
    let tree = expression_support::parse(b"main(){PrivateTarget[3]=4+5;}", mode);
    let LegacyStatementKind::Assignment { target, value } = statement_support::first(&tree).kind()
    else {
        return Err("Expected assignment".into());
    };
    assert_eq!(
        expression_support::bytes(&tree, target),
        b"PrivateTarget[3]"
    );
    assert_eq!(expression_support::bytes(&tree, value), b"4+5");
    for (text, expected) in [
        (b"main(){x++;}".as_slice(), LegacyUpdateKind::Increment),
        (b"main(){x--;}", LegacyUpdateKind::Decrement),
    ] {
        let tree = expression_support::parse(text, mode);
        let LegacyStatementKind::Update { target, kind } = statement_support::first(&tree).kind()
        else {
            return Err("Expected postfix update".into());
        };
        assert_eq!(kind, expected);
        assert_eq!(expression_support::bytes(&tree, target), b"x");
    }
    for (text, expected) in [
        (b"main(){return 123;}".as_slice(), Some(b"123".as_slice())),
        (b"main(){return;}", None),
    ] {
        let tree = expression_support::parse(text, mode);
        let LegacyStatementKind::Return { value } = statement_support::first(&tree).kind() else {
            return Err("Expected return".into());
        };
        assert_eq!(
            value.map(|node| expression_support::bytes(&tree, node)),
            expected
        );
    }
    let tree = expression_support::parse(b"main(){wait 1.5;}", mode);
    let LegacyStatementKind::Wait { duration } = statement_support::first(&tree).kind() else {
        return Err("Expected wait".into());
    };
    assert_eq!(expression_support::bytes(&tree, duration), b"1.5");
    for text in [
        b"main(){PrivateCall(1);}".as_slice(),
        b"main(){self thread PrivateCall();}",
    ] {
        let tree = expression_support::parse(text, mode);
        let LegacyStatementKind::Call { invocation } = statement_support::first(&tree).kind()
        else {
            return Err("Expected call".into());
        };
        let call = LegacyCallSyntax::from_node(invocation).expect("Composable invocation");
        assert!(call.span().start < call.span().end);
        assert!(!format!("{call:?}").contains("PrivateCall"));
    }
    Ok(())
}

#[test]
fn all_ten_compound_assignments_preserve_operator_target_and_rhs() -> Result<(), String> {
    for (spelling, expected) in [
        ("|=", LegacyInfixOperator::BitwiseOr),
        ("^=", LegacyInfixOperator::BitwiseExclusiveOr),
        ("&=", LegacyInfixOperator::BitwiseAnd),
        ("<<=", LegacyInfixOperator::ShiftLeft),
        (">>=", LegacyInfixOperator::ShiftRight),
        ("+=", LegacyInfixOperator::Add),
        ("-=", LegacyInfixOperator::Subtract),
        ("*=", LegacyInfixOperator::Multiply),
        ("/=", LegacyInfixOperator::Divide),
        ("%=", LegacyInfixOperator::Remainder),
    ] {
        let text = format!("main(){{PrivateTarget {spelling} 3;}}");
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Script);
        let view = statement_support::first(&tree);
        let LegacyStatementKind::CompoundAssignment {
            target,
            operator,
            value,
        } = view.kind()
        else {
            return Err("Expected compound assignment".into());
        };
        assert_eq!(operator, expected);
        assert_eq!(expression_support::bytes(&tree, target), b"PrivateTarget");
        assert_eq!(expression_support::bytes(&tree, value), b"3");
        assert!(!format!("{view:?}").contains("PrivateTarget"));
    }
    Ok(())
}
