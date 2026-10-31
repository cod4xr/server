// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structural source-table replay; no original semantic actions or VM execution.
#![allow(
    clippy::expect_used,
    reason = "Tests decode known synthetic grammar fixtures."
)]

use cod4x_script::lexical::LegacyLexErrorKind;
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{
    LegacySyntaxBudget, LegacySyntaxErrorKind, LegacySyntaxMode, LegacySyntaxNodeKind,
    LegacySyntaxTree,
};

fn hex(text: &str) -> Vec<u8> {
    if text == "~" {
        return Vec::new();
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16).expect("Byte")
        })
        .collect()
}

#[test]
fn source_table_acceptance_and_ordered_reductions_match() -> Result<(), String> {
    let mut count = 0;
    for row in include_str!("fixtures/syntax_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let bytes = hex(fields[1]);
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Source");
        let mode = if fields[0] == "script" {
            LegacySyntaxMode::Script
        } else {
            LegacySyntaxMode::Fragment
        };
        let result =
            LegacySyntaxTree::parse_structure(&source, mode, LegacySyntaxBudget::default());
        match (fields[2], result) {
            ("ok", Ok(tree)) => {
                let reductions: Vec<_> = tree
                    .nodes()
                    .filter_map(|node| match node.kind() {
                        LegacySyntaxNodeKind::Production(rule) => Some(rule.as_u16()),
                        _ => None,
                    })
                    .collect();
                let expected: Vec<u16> = fields[3]
                    .split(',')
                    .map(|rule| rule.parse().expect("Rule"))
                    .collect();
                assert_eq!(reductions, expected, "Fixture {count}");
                assert_eq!(tree.source_bytes(), bytes);
                assert_eq!(tree.usage().reductions, reductions.len() as u64);
                assert!(matches!(
                    tree.root().kind(),
                    LegacySyntaxNodeKind::Production(_)
                ));
            }
            ("syntax", Err(error)) => assert_eq!(
                error.kind,
                LegacySyntaxErrorKind::UnexpectedToken,
                "Fixture {count}"
            ),
            ("comment", Err(error)) => assert_eq!(
                error.kind,
                LegacySyntaxErrorKind::Lexical(LegacyLexErrorKind::UnterminatedComment)
            ),
            ("invalid", Err(error)) => assert_eq!(
                error.kind,
                LegacySyntaxErrorKind::Lexical(LegacyLexErrorKind::InvalidByte)
            ),
            (_, result) => {
                return Err(format!(
                    "Unexpected result for structural fixture {count}: {result:?}"
                ));
            }
        }
        count += 1;
    }
    assert_eq!(count, 99);
    Ok(())
}
