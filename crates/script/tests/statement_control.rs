// SPDX-License-Identifier: AGPL-3.0-or-later
//! Control-flow views preserve source roles without resolving scope or executing them.
#![allow(
    clippy::expect_used,
    reason = "Tests provide valid synthetic control statements."
)]

use cod4x_script::statement::{
    LegacyStatementKind, LegacyStatementListSyntax, LegacyStatementSyntax, StatementListBudget,
};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNodeKind};

use crate::expression_support;
use crate::statement_support;

#[test]
fn conditional_and_while_views_preserve_condition_and_separate_branches() -> Result<(), String> {
    for (source, expected_else) in [
        ("main(){if(PrivateCondition)return 1;}", None),
        (
            "main(){if(PrivateCondition)return 1;else return 2;}",
            Some(b"return 2;".as_slice()),
        ),
    ] {
        let tree = expression_support::parse(source.as_bytes(), LegacySyntaxMode::Script);
        let LegacyStatementKind::If {
            condition,
            consequence,
            alternative,
        } = statement_support::first(&tree).kind()
        else {
            return Err("Expected conditional".into());
        };
        assert_eq!(
            expression_support::bytes(&tree, condition),
            b"PrivateCondition"
        );
        assert_eq!(expression_support::bytes(&tree, consequence), b"return 1;");
        assert_eq!(
            alternative.map(|node| expression_support::bytes(&tree, node)),
            expected_else
        );
    }
    let tree = expression_support::parse(
        b"main(){while(PrivateCondition){break;}}",
        LegacySyntaxMode::Script,
    );
    let LegacyStatementKind::While { condition, body } = statement_support::first(&tree).kind()
    else {
        return Err("Expected while".into());
    };
    assert_eq!(
        expression_support::bytes(&tree, condition),
        b"PrivateCondition"
    );
    assert_eq!(expression_support::bytes(&tree, body), b"{break;}");
    assert!(matches!(
        LegacyStatementSyntax::from_node(body)
            .expect("Nested statement")
            .kind(),
        LegacyStatementKind::Block { .. }
    ));
    Ok(())
}

#[test]
fn for_roles_preserve_initializer_and_distinct_optional_condition_update_syntax()
-> Result<(), String> {
    for (source, init, expected_condition, update, condition_rule, update_rule) in [
        (
            "main(){for(x=0;x<3;x++){continue;}}",
            b"x=0;".as_slice(),
            b"x<3".as_slice(),
            b"x++".as_slice(),
            26,
            96,
        ),
        (
            "main(){for(;;){}}",
            b";".as_slice(),
            b"".as_slice(),
            b"".as_slice(),
            27,
            95,
        ),
    ] {
        let tree = expression_support::parse(source.as_bytes(), LegacySyntaxMode::Script);
        let LegacyStatementKind::For {
            initializer,
            condition,
            update: update_node,
            body,
        } = statement_support::first(&tree).kind()
        else {
            return Err("Expected for".into());
        };
        assert_eq!(expression_support::bytes(&tree, initializer), init);
        assert_eq!(
            expression_support::bytes(&tree, condition),
            expected_condition
        );
        assert_eq!(expression_support::bytes(&tree, update_node), update);
        assert!(
            matches!(condition.kind(), LegacySyntaxNodeKind::Production(rule) if rule.as_u16() == condition_rule)
        );
        assert!(
            matches!(update_node.kind(), LegacySyntaxNodeKind::Production(rule) if rule.as_u16() == update_rule)
        );
        LegacyStatementSyntax::from_node(initializer).expect("Initializer statement");
        LegacyStatementSyntax::from_node(body).expect("Loop body statement");
        if update_rule == 96 {
            LegacyStatementSyntax::from_node(update_node).expect("Wrapped update statement");
        }
    }
    Ok(())
}

#[test]
fn switch_labels_developer_blocks_and_empty_statements_stay_separate() -> Result<(), String> {
    let tree = expression_support::parse(
        b"main(){switch(x){case 1:break;default:;}}",
        LegacySyntaxMode::Script,
    );
    let LegacyStatementKind::Switch { expression, body } = statement_support::first(&tree).kind()
    else {
        return Err("Expected switch".into());
    };
    assert_eq!(expression_support::bytes(&tree, expression), b"x");
    let list = LegacyStatementListSyntax::lower(body, StatementListBudget::default())
        .expect("Switch body");
    let kinds = list
        .statements()
        .map(|node| {
            LegacyStatementSyntax::from_node(node)
                .expect("Switch member")
                .kind()
        })
        .collect::<Vec<_>>();
    assert_eq!(kinds.len(), 4);
    let LegacyStatementKind::Case { value } = kinds[0] else {
        return Err("Expected case".into());
    };
    assert_eq!(expression_support::bytes(&tree, value), b"1");
    assert!(matches!(kinds[1], LegacyStatementKind::Break));
    assert!(matches!(kinds[2], LegacyStatementKind::Default));
    assert!(matches!(kinds[3], LegacyStatementKind::Empty));
    let tree = expression_support::parse(b"main(){/# return; #/}", LegacySyntaxMode::Script);
    let LegacyStatementKind::DeveloperBlock { body } = statement_support::first(&tree).kind()
    else {
        return Err("Expected developer block".into());
    };
    let list = LegacyStatementListSyntax::lower(body, StatementListBudget::default())
        .expect("Developer body retained without activation");
    assert_eq!(list.usage().statements, 1);
    Ok(())
}
