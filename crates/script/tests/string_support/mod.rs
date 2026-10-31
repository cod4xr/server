// SPDX-License-Identifier: AGPL-3.0-or-later
//! Constructs owned synthetic strings through the actual source/literal stages.

use cod4x_script::lexical::{LegacyLexBudget, LegacyLexer};
use cod4x_script::literal::{LegacyLiteral, LegacyLiteralBudget, LegacyString, LegacyStringKind};
use cod4x_script::source::{SourceBudget, SourceText};

pub fn decoded(bytes: &[u8], kind: LegacyStringKind) -> Result<LegacyString, String> {
    let mut encoded = Vec::new();
    if kind == LegacyStringKind::Localized {
        encoded.push(b'&');
    }
    encoded.push(b'"');
    for byte in bytes {
        match byte {
            b'"' => encoded.extend_from_slice(b"\\\""),
            b'\\' => encoded.extend_from_slice(b"\\\\"),
            b'\n' => encoded.extend_from_slice(b"\\n"),
            b'\r' => encoded.extend_from_slice(b"\\r"),
            b'\t' => encoded.extend_from_slice(b"\\t"),
            byte => encoded.push(*byte),
        }
    }
    encoded.push(b'"');
    let source = SourceText::index(&encoded, SourceBudget::default())
        .map_err(|_| "Synthetic source admission failed")?;
    let mut lexer = LegacyLexer::new(&source, LegacyLexBudget::default());
    let lexeme = lexer
        .next()
        .ok_or("Expected synthetic string")?
        .map_err(|_| "Synthetic lexing failed")?;
    if lexer.next().is_some() {
        return Err("Expected one synthetic string literal".to_owned());
    }
    let budget = LegacyLiteralBudget {
        max_encoded_bytes: encoded.len() as u64,
        max_string_bytes: bytes.len() as u64,
    };
    match LegacyLiteral::decode(lexeme, budget).map_err(|_| "Synthetic literal decoding failed")? {
        LegacyLiteral::String(value) => Ok(value),
        _ => Err("Expected owned synthetic string".to_owned()),
    }
}
