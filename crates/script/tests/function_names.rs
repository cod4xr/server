// SPDX-License-Identifier: AGPL-3.0-or-later
//! Name/reference syntax preserves spelling and borrowing without linking functions.
#![allow(
    clippy::expect_used,
    reason = "Tests parse exact synthetic function-reference and named-call sources."
)]

use crate::expression_support;

use cod4x_script::expression::{
    LegacyCallSyntax, LegacyCalleeKind, LegacyExpressionError, LegacyFunctionNameSyntax,
    LegacyPrimaryKind, LegacyPrimarySyntax,
};
use cod4x_script::lexical::LegacyLexeme;
use cod4x_script::syntax::LegacySyntaxMode;

#[test]
fn local_and_external_function_references_preserve_module_name_and_original_spans()
-> Result<(), String> {
    for (text, module) in [
        ("::PrivateFunction", None),
        ("PrivateModule::PrivateFunction", Some("PrivateModule")),
        (
            "maps\\mp\\PrivateModule::PrivateFunction",
            Some("maps\\mp\\PrivateModule"),
        ),
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let name = LegacyFunctionNameSyntax::from_node(tree.root()).expect("Function reference");
        assert_eq!(
            name.module().map(LegacyLexeme::as_bytes),
            module.map(str::as_bytes)
        );
        assert_eq!(name.name().as_bytes(), b"PrivateFunction");
        assert_eq!(name.span(), tree.root().span());
        assert_eq!(
            expression_support::bytes(&tree, name.syntax()),
            text.as_bytes()
        );
        let primary = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        let LegacyPrimaryKind::FunctionReference { syntax } = primary.kind() else {
            return Err("Expected function-reference primary".to_owned());
        };
        let direct = LegacyFunctionNameSyntax::from_node(syntax).expect("Direct name node");
        assert_eq!(direct.name().as_bytes(), name.name().as_bytes());
        assert!(!format!("{name:?} {direct:?}").contains("Private"));
    }
    Ok(())
}

#[test]
fn long_callee_names_are_borrowed_without_a_native_scratch_or_identifier_width_ceiling()
-> Result<(), String> {
    let identifier = "Fn".repeat(50_000);
    let text = identifier.clone() + "()";
    let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
    let call = LegacyCallSyntax::from_node(tree.root()).expect("Call");
    let LegacyCalleeKind::Named(name) = call.callee() else {
        return Err("Expected named callee".to_owned());
    };
    assert!(name.module().is_none());
    assert_eq!(name.name().as_bytes(), identifier.as_bytes());
    assert_eq!(name.name().as_bytes().as_ptr(), text.as_bytes().as_ptr());
    assert_eq!(name.span(), 0..identifier.len() as u64);
    assert_eq!(
        LegacyFunctionNameSyntax::from_node(name.syntax())
            .expect("Direct callee")
            .name()
            .as_bytes(),
        identifier.as_bytes()
    );
    assert_eq!(call.span(), 0..text.len() as u64);
    Ok(())
}

#[test]
fn variables_operators_and_whole_invocations_are_not_mistaken_for_named_callee_nodes() {
    for text in [
        "private_variable",
        "1+2",
        "private_call()",
        "[[private_pointer]]()",
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let error = LegacyFunctionNameSyntax::from_node(tree.root()).expect_err("Not a name node");
        assert_eq!(error, LegacyExpressionError::NotFunctionName);
        assert!(!format!("{error:?} {error}").contains("private"));
    }
}
