// SPDX-License-Identifier: AGPL-3.0-or-later
//! Replay source tables/actions; these are not original parser/executable runs.
#![allow(
    clippy::expect_used,
    reason = "Tests decode checked-in source-derived cases."
)]

use cod4x_script::lexical::{LegacyLexBudget, LegacyLexErrorKind, LegacyLexer};
use cod4x_script::source::{SourceBudget, SourceText};

fn hex(text: &str) -> Vec<u8> {
    if text == "~" {
        return Vec::new();
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16).expect("Byte")
        })
        .collect()
}

#[test]
fn exact_rule_spans_trivia_and_terminal_outcomes_match_source_table_replay() {
    let mut count = 0;
    for row in include_str!("fixtures/lexical_vectors.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let row: Vec<_> = row.split_ascii_whitespace().collect();
        let bytes = hex(row[0]);
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Source");
        let mut lexer = LegacyLexer::new(&source, LegacyLexBudget::default());
        let mut actual = Vec::new();
        let mut outcome = "ok";
        for result in lexer.by_ref() {
            match result {
                Ok(lexeme) => {
                    let span = lexeme.span();
                    actual.push(format!(
                        "{}:{}:{}",
                        lexeme.source_rule(),
                        span.start,
                        span.end - span.start
                    ));
                    assert_eq!(
                        lexeme.as_bytes(),
                        source.span(span).expect("Original bytes")
                    );
                }
                Err(error) => {
                    outcome = match error.kind {
                        LegacyLexErrorKind::InvalidByte => "invalid",
                        LegacyLexErrorKind::NulByte => "nul",
                        LegacyLexErrorKind::UnterminatedComment => "comment",
                        _ => return_unexpected(error.kind),
                    };
                }
            }
        }
        assert!(lexer.next().is_none());
        let actual = if actual.is_empty() {
            "~".to_owned()
        } else {
            actual.join(",")
        };
        assert_eq!(actual, row[1], "source hex {}", row[0]);
        assert_eq!(outcome, row[2], "source hex {}", row[0]);
        count += 1;
    }
    assert_eq!(count, 724);
}

fn return_unexpected(kind: LegacyLexErrorKind) -> &'static str {
    // A distinct outcome makes the checked-in terminal comparison report this
    // category rather than hiding an invariant/budget failure in malformed input.
    match kind {
        LegacyLexErrorKind::InvalidTable => "table",
        _ => "budget",
    }
}
