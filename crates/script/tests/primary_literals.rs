// SPDX-License-Identifier: AGPL-3.0-or-later
//! Signed literal syntax stays separate from conversion, binding, and runtime values.
#![allow(
    clippy::expect_used,
    reason = "Tests supply synthetic literal/primary sources and scoped nodes."
)]

use crate::expression_support;

use cod4x_script::expression::{
    LegacyExpressionError, LegacyLiteralPolarity, LegacyPrimaryKind, LegacyPrimarySyntax,
};
use cod4x_script::syntax::LegacySyntaxMode;

#[test]
fn literal_spellings_and_polarity_preserve_original_bytes_without_value_conversion()
-> Result<(), String> {
    for (text, spelling, polarity) in [
        ("123", "123", LegacyLiteralPolarity::Unchanged),
        ("1.25", "1.25", LegacyLiteralPolarity::Unchanged),
        ("-123", "123", LegacyLiteralPolarity::Negated),
        ("-0.0", "0.0", LegacyLiteralPolarity::Negated),
        ("-2147483648", "2147483648", LegacyLiteralPolarity::Negated),
        (
            "99999999999999999999",
            "99999999999999999999",
            LegacyLiteralPolarity::Unchanged,
        ),
        (
            "\"private\\nvalue\"",
            "\"private\\nvalue\"",
            LegacyLiteralPolarity::Unchanged,
        ),
        (
            "&\"private_localized\"",
            "&\"private_localized\"",
            LegacyLiteralPolarity::Unchanged,
        ),
        ("undefined", "undefined", LegacyLiteralPolarity::Unchanged),
        ("false", "false", LegacyLiteralPolarity::Unchanged),
        ("true", "true", LegacyLiteralPolarity::Unchanged),
    ] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let view = LegacyPrimarySyntax::from_node(tree.root()).expect("Primary");
        let LegacyPrimaryKind::Literal {
            token,
            polarity: actual,
        } = view.kind()
        else {
            return Err("Expected literal syntax".to_owned());
        };
        assert_eq!(actual, polarity);
        assert_eq!(token.as_bytes(), spelling.as_bytes());
        assert_eq!(view.span(), tree.root().span());
        assert_eq!(
            expression_support::bytes(&tree, view.syntax()),
            text.as_bytes()
        );
        assert!(!format!("{view:?}").contains("private"));
    }
    Ok(())
}

#[test]
fn operators_and_other_node_categories_fail_without_losing_their_original_tree() {
    for text in ["1+2", "!true", "true&&false"] {
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Fragment);
        let error = LegacyPrimarySyntax::from_node(tree.root()).expect_err("Not primary");
        assert_eq!(error, LegacyExpressionError::NotPrimary);
        assert!(!format!("{error:?} {error}").contains(text));
        assert_eq!(
            expression_support::bytes(&tree, tree.root()),
            text.as_bytes()
        );
    }
    let tree = expression_support::parse(b"private_main(){}", LegacySyntaxMode::Script);
    assert_eq!(
        LegacyPrimarySyntax::from_node(tree.root()).expect_err("Not expression"),
        LegacyExpressionError::NotExpression
    );
}
