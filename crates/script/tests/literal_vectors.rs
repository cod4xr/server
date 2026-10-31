// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source string-action/CRT replays, not original parser or interner execution.
#![allow(
    clippy::expect_used,
    reason = "Tests decode known synthetic literal fixtures."
)]

use cod4x_script::legacy::LegacyNumber;
use cod4x_script::lexical::{LegacyLexBudget, LegacyLexer};
use cod4x_script::literal::{LegacyLiteral, LegacyLiteralBudget, LegacyStringKind};
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
fn string_action_and_defined_crt_numeric_results_match_with_exact_bits() -> Result<(), String> {
    let mut count = 0;
    for row in include_str!("fixtures/literal_vectors.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let row: Vec<_> = row.split_ascii_whitespace().collect();
        let bytes = if row[0] == "S" {
            hex(row[1])
        } else {
            row[2].as_bytes().to_vec()
        };
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Source");
        let mut lexer = LegacyLexer::new(&source, LegacyLexBudget::default());
        let lexeme = lexer.next().expect("One literal").expect("Lexeme");
        assert!(lexer.next().is_none());
        let literal =
            LegacyLiteral::decode(lexeme, LegacyLiteralBudget::default()).expect("Literal");
        match (row[0], literal) {
            ("S", LegacyLiteral::String(value)) => {
                assert_eq!(
                    value.kind(),
                    if row[2] == "plain" {
                        LegacyStringKind::Plain
                    } else {
                        LegacyStringKind::Localized
                    }
                );
                assert_eq!(value.as_bytes(), hex(row[3]));
            }
            ("N", LegacyLiteral::Number(value)) => {
                let bits = u32::from_str_radix(row[3], 16).expect("Number bits");
                match (row[1], value) {
                    ("I", LegacyNumber::Integer(value)) => {
                        assert_eq!(value.to_le_bytes(), bits.to_le_bytes());
                    }
                    ("F", LegacyNumber::Float(value)) => {
                        assert_eq!(value.to_bits(), bits, "{}", row[2]);
                    }
                    _ => return Err("Unexpected numeric literal type".to_owned()),
                }
            }
            _ => return Err("Unexpected decoded literal category".to_owned()),
        }
        count += 1;
    }
    assert_eq!(count, 545);
    Ok(())
}
