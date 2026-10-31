// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original conversion function with explicit ASCII/C-locale and stubbed services.
#![allow(
    clippy::expect_used,
    reason = "Tests decode known synthetic original-function fixtures."
)]

use cod4x_script::binding::LegacyIdentifier;
use cod4x_script::lexical::{LegacyLexBudget, LegacyLexer};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn canonical_identity_matches_original_conversion_across_scratch_fallback_boundaries() {
    let mut count = 0;
    for row in include_str!("fixtures/identifier_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let label: usize = fields[1].parse().expect("Pattern/ASCII");
        let length: usize = fields[2].parse().expect("Length");
        let mut bytes = vec![b'N'; length];
        if fields[0] == "S" {
            bytes[0] = u8::try_from(label).expect("ASCII");
        } else {
            for (index, byte) in bytes.iter_mut().enumerate().skip(1) {
                *byte = b"AaZz_19"[(index + label) % 7];
            }
        }
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Source");
        let mut lexer = LegacyLexer::new(&source, LegacyLexBudget::default());
        let name = LegacyIdentifier::from_lexeme(lexer.next().expect("One name").expect("Lexeme"))
            .expect("Identifier");
        assert!(lexer.next().is_none());
        let digest = name
            .canonical_bytes()
            .fold(14_695_981_039_346_656_037_u64, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211)
            });
        assert_eq!(
            digest,
            u64::from_str_radix(fields[3], 16).expect("Reference checksum")
        );
        assert_eq!(name.is_case_folded(), fields[4] == "1");
        assert_eq!(name.as_bytes().as_ptr(), bytes.as_ptr());
        assert_eq!(name.canonical_bytes().len(), length);
        count += 1;
    }
    assert_eq!(count, 125);
}
