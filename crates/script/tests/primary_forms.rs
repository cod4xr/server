// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local primary roles retain unresolved ordered descendants and private spelling.
#![allow(
    clippy::expect_used,
    reason = "Tests own structurally valid synthetic primary inputs."
)]

mod expression_support;

use std::collections::BTreeSet;

use cod4x_script::expression::{
    LegacyExpressionKind, LegacyExpressionSyntax, LegacyObjectRoot, LegacyPrimaryKind,
    LegacyPrimarySyntax,
};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNodeKind};

#[test]
fn every_primitive_production_has_a_local_typed_role_without_running_descendants()
-> Result<(), String> {
    let mut seen = BTreeSet::new();
    for text in [
        "(1)",
        "1",
        "1.0",
        "-1",
        "-1.0",
        "\"x\"",
        "&\"x\"",
        "f()",
        "x",
        "undefined",
        "self",
        "level",
        "game",
        "anim",
        "x.size",
        "::f",
        "[]",
        "%x",
        "false",
        "true",
        "#animtree",
        "self breakon(x)",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let expression = LegacyExpressionSyntax::from_node(tree.root()).expect("Expression");
        let LegacyExpressionKind::Primary { syntax } = expression.kind() else {
            return Err("Expected structural primary".to_owned());
        };
        let LegacySyntaxNodeKind::Production(rule) = syntax.kind() else {
            return Err("Expected primitive production".to_owned());
        };
        seen.insert(rule.as_u16());
        let view = LegacyPrimarySyntax::from_node(syntax).expect("Typed primary");
        assert_eq!(
            expression_support::bytes(&tree, view.syntax()),
            text.as_bytes()
        );
    }
    assert_eq!(seen, (40..=61).collect());
    Ok(())
}

#[test]
fn parentheses_preserve_the_expression_list_instead_of_guessing_group_or_vector_arity()
-> Result<(), String> {
    for (text, inner) in [("(x)", "x"), ("(1,2,3)", "1,2,3")] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        let LegacyPrimaryKind::Parenthesized { expressions } = view.kind() else {
            return Err("Expected parenthesized list".to_owned());
        };
        assert_eq!(
            expression_support::bytes(&tree, expressions),
            inner.as_bytes()
        );
    }
    Ok(())
}

#[test]
fn calls_and_references_preserve_opaque_subtrees_without_name_or_target_resolution()
-> Result<(), String> {
    for text in [
        "private_name",
        "$private_name",
        "self.private_field",
        "private_array[1]",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        let LegacyPrimaryKind::Reference { syntax } = view.kind() else {
            return Err("Expected reference".to_owned());
        };
        assert_eq!(expression_support::bytes(&tree, syntax), text.as_bytes());
        assert!(!format!("{view:?}").contains("private"));
    }
    for text in [
        "private_call(1)",
        "self thread private_call()",
        "[[private_callback]](1)",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        let LegacyPrimaryKind::Call { syntax } = view.kind() else {
            return Err("Expected invocation".to_owned());
        };
        assert_eq!(expression_support::bytes(&tree, syntax), text.as_bytes());
    }
    for text in [
        "::PrivateFunction",
        "maps\\mp\\PrivateModule::PrivateFunction",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        let LegacyPrimaryKind::FunctionReference { syntax } = view.kind() else {
            return Err("Expected function reference".to_owned());
        };
        assert_eq!(expression_support::bytes(&tree, syntax), text.as_bytes());
    }
    Ok(())
}

#[test]
fn object_roots_size_arrays_and_animation_forms_create_no_runtime_objects() -> Result<(), String> {
    for (text, expected) in [
        ("self", LegacyObjectRoot::CurrentObject),
        ("level", LegacyObjectRoot::Level),
        ("game", LegacyObjectRoot::Game),
        ("anim", LegacyObjectRoot::Animation),
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        assert!(matches!(view.kind(), LegacyPrimaryKind::Object(actual) if actual == expected));
    }
    let tree = expression_support::parse(b"private_values.size", LegacySyntaxMode::Fragment);
    let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
    let LegacyPrimaryKind::Size { object } = view.kind() else {
        return Err("Expected size access".to_owned());
    };
    assert_eq!(expression_support::bytes(&tree, object), b"private_values");
    let array = expression_support::parse(b"[]", LegacySyntaxMode::Fragment);
    assert!(matches!(
        LegacyPrimarySyntax::from_node(array.root())
            .expect("Array")
            .kind(),
        LegacyPrimaryKind::EmptyArray
    ));
    let animation = expression_support::parse(b"%PrivateAnimation", LegacySyntaxMode::Fragment);
    let view = LegacyPrimarySyntax::from_node(animation.root()).expect("Animation");
    let LegacyPrimaryKind::Animation { name } = view.kind() else {
        return Err("Expected named animation".to_owned());
    };
    assert_eq!(name.as_bytes(), b"PrivateAnimation");
    let animation_tree = expression_support::parse(b"#animtree", LegacySyntaxMode::Fragment);
    assert!(matches!(
        LegacyPrimarySyntax::from_node(animation_tree.root())
            .expect("Animation tree")
            .kind(),
        LegacyPrimaryKind::AnimationTree
    ));
    Ok(())
}

#[test]
fn breakon_retains_its_original_object_and_condition_without_enabling_debugger_services()
-> Result<(), String> {
    let tree = expression_support::parse(b"self breakon(1+2)", LegacySyntaxMode::Fragment);
    let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
    let LegacyPrimaryKind::BreakOn { object, condition } = view.kind() else {
        return Err("Expected breakon syntax".to_owned());
    };
    assert_eq!(expression_support::bytes(&tree, object), b"self");
    assert_eq!(expression_support::bytes(&tree, condition), b"1+2");
    let nested = LegacyExpressionSyntax::from_node(condition).expect("Condition expression");
    assert!(matches!(nested.kind(), LegacyExpressionKind::Infix { .. }));
    Ok(())
}
