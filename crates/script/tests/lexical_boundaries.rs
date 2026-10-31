// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded lexing, exact spans, pinned terminal codes, and explicit failure closure.
#![allow(
    clippy::expect_used,
    reason = "Tests construct exact source and lexer budgets."
)]

use cod4x_script::lexical::{
    LegacyLexBudget, LegacyLexErrorKind, LegacyLexResource, LegacyLexemeKind, LegacyLexer,
};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn grammar_codes_and_identifier_case_remain_raw_without_new_keywords_or_interning() {
    let source = SourceText::index(
        b"if foreach Foo 1 1.5 \"private-string\" &\"localized\"",
        SourceBudget::default(),
    )
    .expect("Source");
    let mut codes = Vec::new();
    for result in LegacyLexer::new(&source, LegacyLexBudget::default()) {
        let lexeme = result.expect("Valid");
        if let LegacyLexemeKind::Token(code) = lexeme.kind() {
            codes.push(code.as_u16());
            assert!(!format!("{lexeme:?}").contains("private"));
            if code.as_u16() == 0x102 && lexeme.as_bytes() == b"Foo" {
                assert_eq!(lexeme.as_bytes(), b"Foo");
            }
        }
    }
    assert_eq!(codes, [0x12f, 0x102, 0x102, 0x11f, 0x120, 0x103, 0x104]);
}

#[test]
fn trivia_and_borrowed_spans_partition_valid_source_without_hidden_prefix_bytes() {
    let bytes = b"/* comment\r\n body */\tmain() { // line\n return; }";
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    let mut end = 0;
    for result in LegacyLexer::new(&source, LegacyLexBudget::default()) {
        let lexeme = result.expect("Valid");
        let span = lexeme.span();
        assert_eq!(span.start, end);
        assert_eq!(
            lexeme.as_bytes().as_ptr(),
            bytes[usize::try_from(end).expect("Offset")..].as_ptr()
        );
        end = span.end;
    }
    assert_eq!(end, bytes.len() as u64);
}

#[test]
fn each_budget_fails_explicitly_and_stops_iteration_permanently() {
    let source = SourceText::index(b"abc ", SourceBudget::default()).expect("Source");
    for (budget, resource) in [
        (
            LegacyLexBudget {
                max_lexemes: 0,
                ..LegacyLexBudget::default()
            },
            LegacyLexResource::Lexemes,
        ),
        (
            LegacyLexBudget {
                max_steps: 0,
                ..LegacyLexBudget::default()
            },
            LegacyLexResource::Steps,
        ),
        (
            LegacyLexBudget {
                max_lexeme_bytes: 2,
                ..LegacyLexBudget::default()
            },
            LegacyLexResource::LexemeBytes,
        ),
    ] {
        let mut lexer = LegacyLexer::new(&source, budget);
        let error = lexer.next().expect("Failure result").expect_err("Limit");
        assert_eq!(error.kind, LegacyLexErrorKind::BudgetExceeded(resource));
        assert!(lexer.next().is_none());
    }
    let mut lexer = LegacyLexer::new(
        &source,
        LegacyLexBudget {
            max_lexeme_bytes: 3,
            max_lexemes: 2,
            ..LegacyLexBudget::default()
        },
    );
    assert_eq!(
        lexer
            .next()
            .expect("Token")
            .expect("Exact bytes")
            .as_bytes(),
        b"abc"
    );
    assert_eq!(
        lexer
            .next()
            .expect("Space")
            .expect("Exact count")
            .as_bytes(),
        b" "
    );
    assert!(lexer.next().is_none());
}

#[test]
fn eof_empty_nul_and_unclosed_comment_have_defined_offsets_and_no_source_disclosure() {
    let empty = SourceText::index(b"", SourceBudget::default()).expect("Empty");
    assert!(
        LegacyLexer::new(
            &empty,
            LegacyLexBudget {
                max_lexemes: 0,
                max_steps: 0,
                max_lexeme_bytes: 0
            }
        )
        .next()
        .is_none()
    );
    let source = SourceText::index(b" \0private", SourceBudget::default()).expect("Raw source");
    let error = LegacyLexer::new(&source, LegacyLexBudget::default())
        .next()
        .expect("Error")
        .expect_err("NUL");
    assert_eq!((error.kind, error.offset), (LegacyLexErrorKind::NulByte, 1));
    assert!(!error.to_string().contains("private"));
    let source = SourceText::index(b"/*unfinished", SourceBudget::default()).expect("Source");
    let error = LegacyLexer::new(&source, LegacyLexBudget::default())
        .last()
        .expect("Error")
        .expect_err("Comment");
    assert_eq!(
        (error.kind, error.offset),
        (LegacyLexErrorKind::UnterminatedComment, 0)
    );
}
