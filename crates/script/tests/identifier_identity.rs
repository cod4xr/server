// SPDX-License-Identifier: AGPL-3.0-or-later
//! Textual identity, hash consistency, source privacy, and wrong-token admission.
#![allow(
    clippy::expect_used,
    reason = "Tests own exact synthetic identifier lexemes."
)]

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use cod4x_script::binding::{LegacyIdentifier, LegacyIdentifierError};
use cod4x_script::lexical::{LegacyLexBudget, LegacyLexemeKind, LegacyLexer};
use cod4x_script::source::{SourceBudget, SourceText};

fn identifier<'source>(source: &SourceText<'source>) -> LegacyIdentifier<'source> {
    let mut lexer = LegacyLexer::new(source, LegacyLexBudget::default());
    let name = LegacyIdentifier::from_lexeme(lexer.next().expect("Name").expect("Lexeme"))
        .expect("Identifier");
    assert!(lexer.next().is_none());
    name
}

fn hash(name: LegacyIdentifier<'_>) -> u64 {
    let mut hash = DefaultHasher::new();
    name.hash(&mut hash);
    hash.finish()
}

#[test]
fn short_names_fold_case_with_hash_equality_independent_of_source_and_offsets() {
    let upper = SourceText::index(b"Private_Name", SourceBudget::default()).expect("Source");
    let lower = SourceText::index(b"private_name", SourceBudget::default()).expect("Source");
    let (upper, lower) = (identifier(&upper), identifier(&lower));
    assert_eq!(upper, lower);
    assert_eq!(hash(upper), hash(lower));
    assert_eq!(HashSet::from([upper, lower]).len(), 1);
    assert_eq!(upper.canonical_bytes().collect::<Vec<_>>(), b"private_name");
    assert!(!format!("{upper:?}").to_lowercase().contains("private_name"));
    assert_eq!(upper.lexeme().span(), 0..12);
    let shifted = SourceText::index(b" \nprivate_name", SourceBudget::default()).expect("Source");
    let mut lexer = LegacyLexer::new(&shifted, LegacyLexBudget::default());
    let lexeme = lexer
        .by_ref()
        .map(|result| result.expect("Lexeme"))
        .find(|lexeme| matches!(lexeme.kind(), LegacyLexemeKind::Token(_)))
        .expect("Name");
    let shifted = LegacyIdentifier::from_lexeme(lexeme).expect("Identifier");
    assert!(lexer.next().is_none());
    assert_eq!(shifted.lexeme().span(), 2..14);
    assert_eq!(upper, shifted);
    assert_eq!(hash(upper), hash(shifted));
}

#[test]
fn long_names_keep_native_exact_case_without_a_storage_ceiling() {
    for length in [8191, 8192, 100_000] {
        let upper_bytes = vec![b'A'; length];
        let lower_bytes = vec![b'a'; length];
        let upper = SourceText::index(&upper_bytes, SourceBudget::default()).expect("Source");
        let lower = SourceText::index(&lower_bytes, SourceBudget::default()).expect("Source");
        let (upper, lower) = (identifier(&upper), identifier(&lower));
        assert_eq!(upper == lower, length < 8192);
        assert_eq!(
            upper.canonical_bytes().next(),
            Some(if length < 8192 { b'a' } else { b'A' })
        );
        assert_eq!(
            upper.canonical_bytes().next_back(),
            upper.canonical_bytes().next()
        );
        assert_eq!(upper.as_bytes().len(), length);
    }
}

#[test]
fn keywords_numbers_strings_and_trivia_are_not_identifier_keys() {
    for bytes in [b"if".as_slice(), b"1", b"\"private-secret\"", b" "] {
        let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
        let lexeme = LegacyLexer::new(&source, LegacyLexBudget::default())
            .next()
            .expect("Lexeme")
            .expect("Scanned");
        let error = LegacyIdentifier::from_lexeme(lexeme).expect_err("Wrong terminal");
        assert_eq!(error, LegacyIdentifierError::WrongToken);
        assert!(!format!("{error:?} {error}").contains("private"));
    }
}
