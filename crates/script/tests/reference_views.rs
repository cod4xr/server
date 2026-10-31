// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local reference roles retain original data without scope/object/mutability policy.
#![allow(
    clippy::expect_used,
    reason = "Tests own exact synthetic reference trees and scoped views."
)]

use crate::expression_support;

use cod4x_script::expression::{
    LegacyCallSyntax, LegacyCalleeKind, LegacyExpressionError, LegacyPrimaryKind,
    LegacyPrimarySyntax, LegacyReferenceKind, LegacyReferenceSyntax,
};
use cod4x_script::syntax::LegacySyntaxMode;

#[test]
fn identifiers_and_dollar_names_preserve_tokens_wrappers_and_private_debug() -> Result<(), String> {
    for text in ["PrivateName", "$PrivateName"] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyReferenceSyntax::from_node(tree.root()).expect("Reference");
        let name = match view.kind() {
            LegacyReferenceKind::Name { name } => {
                assert_eq!(text, "PrivateName");
                name
            }
            LegacyReferenceKind::DollarName { name } => {
                assert_eq!(text, "$PrivateName");
                name
            }
            _ => return Err("Expected original name form".to_owned()),
        };
        assert_eq!(name.as_bytes(), b"PrivateName");
        assert_eq!(view.span(), tree.root().span());
        assert_eq!(
            expression_support::bytes(&tree, view.syntax()),
            text.as_bytes()
        );
        let primary = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        let LegacyPrimaryKind::Reference { syntax } = primary.kind() else {
            return Err("Expected reference primary".to_owned());
        };
        let direct = LegacyReferenceSyntax::from_node(syntax).expect("Direct reference");
        assert_eq!(direct.span(), syntax.span());
        assert!(!format!("{view:?} {direct:?}").contains("PrivateName"));
        assert_eq!(tree.source_bytes(), text.as_bytes());
    }
    Ok(())
}

#[test]
fn nested_field_index_and_call_operands_remain_ordered_unresolved_syntax() -> Result<(), String> {
    let text = b"private_players[get_index()].Health";
    let tree = expression_support::parse(text, LegacySyntaxMode::Fragment);
    let field = LegacyReferenceSyntax::from_node(tree.root()).expect("Field");
    let LegacyReferenceKind::Field { object, name } = field.kind() else {
        return Err("Expected field access".to_owned());
    };
    assert_eq!(name.as_bytes(), b"Health");
    assert_eq!(
        expression_support::bytes(&tree, object),
        b"private_players[get_index()]"
    );
    let indexed = LegacyReferenceSyntax::from_node(object).expect("Indexed object");
    let LegacyReferenceKind::Index { object, index } = indexed.kind() else {
        return Err("Expected indexed reference".to_owned());
    };
    assert_eq!(expression_support::bytes(&tree, object), b"private_players");
    assert_eq!(expression_support::bytes(&tree, index), b"get_index()");
    let base = LegacyReferenceSyntax::from_node(object).expect("Base name");
    assert!(
        matches!(base.kind(), LegacyReferenceKind::Name { name } if name.as_bytes() == b"private_players")
    );
    let call = LegacyCallSyntax::from_node(index).expect("Index call shape");
    let LegacyCalleeKind::Named(name) = call.callee() else {
        return Err("Expected named index callee".to_owned());
    };
    assert_eq!(name.name().as_bytes(), b"get_index");
    assert_eq!(tree.source_bytes(), text);
    Ok(())
}

#[test]
fn reserved_self_member_and_ordinary_case_variant_fields_remain_distinct() -> Result<(), String> {
    let tree = expression_support::parse(b"private_object.self", LegacySyntaxMode::Fragment);
    let view = LegacyReferenceSyntax::from_node(tree.root()).expect("Reserved member");
    let LegacyReferenceKind::SelfMember { object } = view.kind() else {
        return Err("Expected reserved self member".to_owned());
    };
    assert_eq!(expression_support::bytes(&tree, object), b"private_object");
    let other = expression_support::parse(b"private_object.Self", LegacySyntaxMode::Fragment);
    let ordinary = LegacyReferenceSyntax::from_node(other.root()).expect("Identifier field");
    let LegacyReferenceKind::Field { object, name } = ordinary.kind() else {
        return Err("Expected ordinary field".to_owned());
    };
    assert_eq!(name.as_bytes(), b"Self");
    assert_eq!(expression_support::bytes(&other, object), b"private_object");
    // Both independently owned source trees and their views remain valid together.
    assert_eq!(
        expression_support::bytes(&tree, view.syntax()),
        b"private_object.self"
    );
    assert!(!format!("{view:?} {ordinary:?}").contains("private"));
    Ok(())
}

#[test]
fn twelve_thousand_indexed_objects_can_be_viewed_without_recursive_ownership() -> Result<(), String>
{
    let depth = 12_000;
    let text = "private_values".to_owned() + &"[0]".repeat(depth);
    let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
    let mut node = tree.root();
    for _ in 0..depth {
        let reference = LegacyReferenceSyntax::from_node(node).expect("Index");
        let LegacyReferenceKind::Index { object, index } = reference.kind() else {
            return Err("Expected indexed chain".to_owned());
        };
        assert_eq!(expression_support::bytes(&tree, index), b"0");
        node = object;
    }
    let base = LegacyReferenceSyntax::from_node(node).expect("Base name");
    assert!(
        matches!(base.kind(), LegacyReferenceKind::Name { name } if name.as_bytes() == b"private_values")
    );
    Ok(())
}

#[test]
fn unrelated_nodes_and_special_object_values_are_not_implicitly_writable_references() {
    for text in [
        "1+2",
        "private_call()",
        "self",
        "private_value.size",
        "::private_function",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let error = LegacyReferenceSyntax::from_node(tree.root()).expect_err("Not a reference");
        assert_eq!(error, LegacyExpressionError::NotReference);
        assert!(!format!("{error:?} {error}").contains("private"));
    }
}
