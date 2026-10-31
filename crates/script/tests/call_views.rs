// SPDX-License-Identifier: AGPL-3.0-or-later
//! Named/indirect, method/function, and immediate/thread roles stay unresolved.
#![allow(
    clippy::expect_used,
    reason = "Tests supply bounded synthetic call syntax and explicit list budgets."
)]

use crate::expression_support;

use cod4x_script::expression::{
    ExpressionListBudget, ExpressionListErrorKind, ExpressionListResource, LegacyCallSyntax,
    LegacyCalleeKind, LegacyExpressionError, LegacyInvocationKind,
};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNodeKind};

#[test]
fn callee_receiver_and_invocation_modes_preserve_source_without_creating_execution() {
    for receiver in ["", "private_receiver "] {
        for (thread, expected_kind) in [
            ("", LegacyInvocationKind::Immediate),
            ("thread ", LegacyInvocationKind::Thread),
        ] {
            for target in ["PrivateFunction", "[[private_callback]]"] {
                let text = format!("{receiver}{thread}{target}(private_arg,nested(2))");
                let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
                let view = LegacyCallSyntax::from_node(tree.root()).expect("Invocation");
                assert_eq!(view.kind(), expected_kind);
                assert_eq!(view.span(), tree.root().span());
                assert_eq!(view.syntax().span(), tree.root().span());
                if receiver.is_empty() {
                    assert!(view.receiver().is_none());
                } else {
                    assert_eq!(
                        expression_support::bytes(&tree, view.receiver().expect("Receiver")),
                        b"private_receiver"
                    );
                }
                match view.callee() {
                    LegacyCalleeKind::Named(name) => {
                        assert_eq!(target, "PrivateFunction");
                        assert!(name.module().is_none());
                        assert_eq!(name.name().as_bytes(), b"PrivateFunction");
                    }
                    LegacyCalleeKind::Indirect { expression } => {
                        assert_eq!(target, "[[private_callback]]");
                        assert_eq!(
                            expression_support::bytes(&tree, expression),
                            b"private_callback"
                        );
                    }
                }
                let arguments = view
                    .lower_arguments(ExpressionListBudget::default())
                    .expect("Argument index");
                let actual: Vec<_> = arguments
                    .expressions()
                    .map(|node| expression_support::bytes(&tree, node))
                    .collect();
                assert_eq!(actual, [b"private_arg".as_slice(), b"nested(2)"]);
                assert_eq!(arguments.span(), view.arguments_syntax().span());
                assert!(!format!("{view:?} {arguments:?}").contains("private"));
                assert_eq!(tree.source_bytes(), text.as_bytes());
            }
        }
    }
}

#[test]
fn external_module_and_identifier_spelling_remain_original_until_binding() -> Result<(), String> {
    for module in ["PrivateModule", "maps\\mp\\PrivateModule"] {
        let text = format!("self thread {module}::PrivateFunction()");
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyCallSyntax::from_node(tree.root()).expect("Call");
        let LegacyCalleeKind::Named(name) = view.callee() else {
            return Err("Expected named external callee".to_owned());
        };
        assert_eq!(
            name.module().expect("Module token").as_bytes(),
            module.as_bytes()
        );
        assert_eq!(name.name().as_bytes(), b"PrivateFunction");
        assert_eq!(
            expression_support::bytes(&tree, name.syntax()),
            format!("{module}::PrivateFunction").as_bytes()
        );
        let direct = tree.nodes().find(|node| matches!(node.kind(), LegacySyntaxNodeKind::Production(rule) if rule.as_u16() == 39)).expect("Direct method node");
        let other = LegacyCallSyntax::from_node(direct).expect("Direct call view");
        assert_eq!(other.span(), direct.span());
        assert_eq!(other.kind(), LegacyInvocationKind::Thread);
    }
    Ok(())
}

#[test]
fn argument_allocation_is_separate_budgeted_work_and_empty_calls_need_no_index_bytes() {
    let tree = expression_support::parse(b"private_call(1,2)", LegacySyntaxMode::Fragment);
    let view = LegacyCallSyntax::from_node(tree.root()).expect("Local call shape");
    let error = view
        .lower_arguments(ExpressionListBudget {
            max_expressions: 0,
            ..ExpressionListBudget::default()
        })
        .expect_err("Separate admission");
    assert_eq!(
        error.kind,
        ExpressionListErrorKind::BudgetExceeded {
            resource: ExpressionListResource::Expressions,
            requested: 1,
            maximum: 0,
        }
    );
    assert_eq!(error.offset, view.arguments_syntax().span().start);
    assert_eq!(view.kind(), LegacyInvocationKind::Immediate);
    assert!(!format!("{error:?} {error}").contains("private"));
    let empty_tree = expression_support::parse(b"empty_call()", LegacySyntaxMode::Fragment);
    let empty = LegacyCallSyntax::from_node(empty_tree.root()).expect("Empty call");
    let arguments = empty
        .lower_arguments(ExpressionListBudget {
            max_expressions: 0,
            max_steps: 2,
            max_metadata_bytes: 0,
        })
        .expect("Zero index storage");
    assert_eq!(arguments.expressions().len(), 0);
    assert_eq!(arguments.usage().metadata_bytes, 0);
    // Independent tree/view and prior arguments are not changed by failed admission.
    assert_eq!(
        expression_support::bytes(&tree, view.arguments_syntax()),
        b"1,2"
    );
}

#[test]
fn noncall_nodes_fail_with_a_fixed_category_instead_of_becoming_invocations() {
    for text in [
        "private_value",
        "1+2",
        "::private_reference",
        "\"private_literal\"",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let error = LegacyCallSyntax::from_node(tree.root()).expect_err("Not a call");
        assert_eq!(error, LegacyExpressionError::NotCall);
        assert!(!format!("{error:?} {error}").contains("private"));
    }
    let tree = expression_support::parse(b"private_function(){}", LegacySyntaxMode::Script);
    assert_eq!(
        LegacyCallSyntax::from_node(tree.root()).expect_err("Declaration"),
        LegacyExpressionError::NotCall
    );
}
