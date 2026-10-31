// SPDX-License-Identifier: AGPL-3.0-or-later
//! Literal domains, capacity expansion, source immutability, and sanitized errors.
#![allow(
    clippy::expect_used,
    reason = "Tests construct known valid lexical inputs."
)]

use cod4x_script::legacy::LegacyNumber;
use cod4x_script::lexical::{LegacyLexBudget, LegacyLexer};
use cod4x_script::literal::{
    LegacyLiteral, LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource,
};
use cod4x_script::source::{SourceBudget, SourceText};

fn decode(bytes: &[u8], budget: LegacyLiteralBudget) -> Result<LegacyLiteral, LegacyLiteralError> {
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Test source");
    let mut lexer = LegacyLexer::new(&source, LegacyLexBudget::default());
    let lexeme = lexer.next().expect("Lexeme").expect("Valid token");
    assert!(
        lexer.next().is_none(),
        "Test input must be exactly one lexeme"
    );
    LegacyLiteral::decode(lexeme, budget)
}

#[test]
fn long_strings_use_configured_storage_without_the_old_scratch_buffer() -> Result<(), String> {
    let mut source = vec![b'"'];
    source.extend(std::iter::repeat_n(b'x', 100_000));
    source.push(b'"');
    let original = source.clone();
    let budget = LegacyLiteralBudget {
        max_encoded_bytes: 100_002,
        max_string_bytes: 100_000,
    };
    let LegacyLiteral::String(value) = decode(&source, budget).expect("Long decoded string") else {
        return Err("Expected a decoded string".to_owned());
    };
    assert_eq!(value.as_bytes().len(), 100_000);
    assert!(value.as_bytes().iter().all(|byte| *byte == b'x'));
    assert_eq!(source, original);
    assert_eq!(
        decode(
            &source,
            LegacyLiteralBudget {
                max_encoded_bytes: 100_001,
                ..budget
            }
        )
        .expect_err("Encoded limit"),
        LegacyLiteralError::BudgetExceeded(LegacyLiteralResource::EncodedBytes)
    );
    assert_eq!(
        decode(
            &source,
            LegacyLiteralBudget {
                max_string_bytes: 99_999,
                ..budget
            }
        )
        .expect_err("Decoded limit"),
        LegacyLiteralError::BudgetExceeded(LegacyLiteralResource::StringBytes)
    );
    Ok(())
}

#[test]
fn legacy_unknown_escapes_empty_strings_and_debug_preserve_defined_behavior() -> Result<(), String>
{
    let LegacyLiteral::String(value) = decode(
        br#""private\q\x41\u0041\0\b\n\r\t\\\"""#,
        LegacyLiteralBudget::default(),
    )
    .expect("Escaped string") else {
        return Err("String".to_owned());
    };
    assert_eq!(value.as_bytes(), b"privateqx41u00410b\n\r\t\\\"");
    assert!(!format!("{value:?}").contains("private"));
    let LegacyLiteral::String(value) = decode(
        b"\"\"",
        LegacyLiteralBudget {
            max_encoded_bytes: 2,
            max_string_bytes: 0,
        },
    )
    .expect("Empty decoded bytes") else {
        return Err("String".to_owned());
    };
    assert!(value.into_bytes().is_empty());
    Ok(())
}

#[test]
fn booleans_undefined_wrong_tokens_and_checked_numeric_domains_are_explicit() {
    assert!(matches!(
        decode(b"undefined", LegacyLiteralBudget::default()),
        Ok(LegacyLiteral::Undefined)
    ));
    for (bytes, expected) in [
        (b"true".as_slice(), 1),
        (b"false".as_slice(), 0),
        (b"2147483647".as_slice(), i32::MAX),
    ] {
        assert!(matches!(decode(bytes, LegacyLiteralBudget::default()),
            Ok(LegacyLiteral::Number(LegacyNumber::Integer(value))) if value == expected));
    }
    for bytes in [
        b"2147483648".as_slice(),
        b"999999999999999999999999",
        b"9999999999999999999999999999999999999999999999999999999999999999.0",
    ] {
        assert_eq!(
            decode(bytes, LegacyLiteralBudget::default()).expect_err("Numeric domain"),
            LegacyLiteralError::NumberOutOfRange
        );
    }
    for bytes in [b"identifier".as_slice(), b" ", b"-"] {
        assert_eq!(
            decode(bytes, LegacyLiteralBudget::default()).expect_err("Not a literal"),
            LegacyLiteralError::WrongToken
        );
    }
}
