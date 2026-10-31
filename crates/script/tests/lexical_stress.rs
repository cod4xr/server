// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hostile-byte closure, long bounded lexemes, and logical work-accounting gates.
#![allow(
    clippy::expect_used,
    reason = "Tests own exact source and boundary budgets."
)]

use cod4x_script::lexical::{LegacyLexBudget, LegacyLexErrorKind, LegacyLexResource, LegacyLexer};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn every_two_byte_input_terminates_without_table_errors_or_invalid_spans() {
    for bits in 0..=u16::MAX {
        let bytes = bits.to_le_bytes();
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Raw source");
        let mut lexer = LegacyLexer::new(
            &source,
            LegacyLexBudget {
                max_lexemes: 4,
                max_steps: 64,
                max_lexeme_bytes: 2,
            },
        );
        let mut end = 0;
        for result in lexer.by_ref() {
            match result {
                Ok(lexeme) => {
                    let span = lexeme.span();
                    assert_eq!(span.start, end);
                    assert!(span.end > span.start && span.end <= 2);
                    end = span.end;
                }
                Err(error) => {
                    assert_ne!(error.kind, LegacyLexErrorKind::InvalidTable);
                    assert!(error.offset <= 2);
                }
            }
        }
        assert!(lexer.next().is_none());
        assert!(lexer.usage().steps <= 64);
    }
}

#[test]
fn long_raw_literals_are_borrowed_under_explicit_budgets_without_literal_decoding() {
    let mut bytes = vec![b'"'];
    bytes.extend(std::iter::repeat_n(b'x', 100_000));
    bytes.push(b'"');
    let source = SourceText::index(&bytes, SourceBudget::default()).expect("Long raw source");
    let mut lexer = LegacyLexer::new(&source, LegacyLexBudget::default());
    let lexeme = lexer
        .next()
        .expect("Literal")
        .expect("Admitted raw literal");
    assert_eq!(lexeme.source_rule(), 7);
    assert_eq!(lexeme.as_bytes().as_ptr(), bytes.as_ptr());
    assert_eq!(lexeme.as_bytes().len(), 100_002);
    assert!(lexer.next().is_none());
    let mut limited = LegacyLexer::new(
        &source,
        LegacyLexBudget {
            max_lexeme_bytes: 100_001,
            ..LegacyLexBudget::default()
        },
    );
    assert_eq!(
        limited.next().expect("Limit").expect_err("Oversize").kind,
        LegacyLexErrorKind::BudgetExceeded(LegacyLexResource::LexemeBytes)
    );
}

#[test]
fn step_and_output_budgets_accept_exact_observed_usage_and_reject_one_less() {
    let source = SourceText::index(b"abc ", SourceBudget::default()).expect("Source");
    let mut lexer = LegacyLexer::new(&source, LegacyLexBudget::default());
    for result in lexer.by_ref() {
        result.expect("Valid");
    }
    let usage = lexer.usage();
    assert_eq!(usage.attempted_lexemes, 2);
    assert_eq!(usage.consumed_bytes, 4);
    assert!(usage.steps > 0);
    let exact = LegacyLexBudget {
        max_lexemes: 2,
        max_steps: usage.steps,
        max_lexeme_bytes: 3,
    };
    assert!(LegacyLexer::new(&source, exact).all(|result| result.is_ok()));
    let error = LegacyLexer::new(
        &source,
        LegacyLexBudget {
            max_steps: usage.steps - 1,
            ..exact
        },
    )
    .find_map(Result::err)
    .expect("One-less work fails");
    assert_eq!(
        error.kind,
        LegacyLexErrorKind::BudgetExceeded(LegacyLexResource::Steps)
    );
    let error = LegacyLexer::new(
        &source,
        LegacyLexBudget {
            max_lexemes: 1,
            ..exact
        },
    )
    .find_map(Result::err)
    .expect("One-less output fails");
    assert_eq!(
        error.kind,
        LegacyLexErrorKind::BudgetExceeded(LegacyLexResource::Lexemes)
    );
}
