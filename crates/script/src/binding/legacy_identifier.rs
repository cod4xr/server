// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::binding::LegacyIdentifierError;
use crate::lexical::{LegacyLexeme, LegacyLexemeKind};

// Native names at/above this length bypass lowercase conversion. This is a
// compatibility quirk, never a storage/admission limit or modern naming policy.
const LEGACY_CASE_BUFFER_BYTES: usize = 8192;

/// Borrowed textual legacy name identity, without interning or a runtime ID.
///
/// Under this explicit ASCII/C-locale profile, names shorter than 8192 bytes fold
/// ASCII case. Longer names retain exact case, matching the original helper's
/// scratch-buffer fallback. All lengths remain admitted by source/lexical budgets;
/// no name is truncated or copied into a fixed buffer. Modern naming is separate.
#[derive(Clone, Copy)]
pub struct LegacyIdentifier<'source> {
    lexeme: LegacyLexeme<'source>,
}

impl<'source> LegacyIdentifier<'source> {
    /// Borrows an already-scanned identifier and verifies the ASCII profile.
    ///
    /// No allocation, interning, case mutation, or engine state change occurs.
    /// Source/scanner budgets are admitted separately; ASCII admission scans this
    /// one name and canonical-byte iteration remains linear in its byte length.
    ///
    /// # Errors
    /// Rejects other token kinds and non-ASCII spellings. Undefined/locale-dependent
    /// native tolower behavior outside this profile is not claimed compatible.
    pub fn from_lexeme(lexeme: LegacyLexeme<'source>) -> Result<Self, LegacyIdentifierError> {
        if !matches!(lexeme.kind(), LegacyLexemeKind::Token(code) if code.as_u16() == 0x102) {
            return Err(LegacyIdentifierError::WrongToken);
        }
        if !lexeme.as_bytes().is_ascii() {
            return Err(LegacyIdentifierError::NonAscii);
        }
        Ok(Self { lexeme })
    }

    /// Borrows the original spelling; these bytes may contain private script names.
    #[must_use]
    pub fn as_bytes(self) -> &'source [u8] {
        self.lexeme.as_bytes()
    }

    /// Returns the original lexeme/span for diagnostics without exposing an ID.
    #[must_use]
    pub fn lexeme(self) -> LegacyLexeme<'source> {
        self.lexeme
    }

    /// Reports the native normalization branch, including already-lowercase names.
    /// False means exact-case identity, not rejected or oversized storage.
    #[must_use]
    pub fn is_case_folded(self) -> bool {
        self.as_bytes().len() < LEGACY_CASE_BUFFER_BYTES
    }

    /// Iterates canonical identity bytes without allocating or adding a terminator.
    #[must_use]
    pub fn canonical_bytes(
        self,
    ) -> impl ExactSizeIterator<Item = u8> + DoubleEndedIterator + 'source {
        let folded = self.is_case_folded();
        self.as_bytes().iter().map(move |byte| {
            if folded {
                byte.to_ascii_lowercase()
            } else {
                *byte
            }
        })
    }
}

impl PartialEq for LegacyIdentifier<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.is_case_folded() {
            self.as_bytes().eq_ignore_ascii_case(other.as_bytes())
        } else {
            self.as_bytes() == other.as_bytes()
        }
    }
}

impl Eq for LegacyIdentifier<'_> {}

impl Hash for LegacyIdentifier<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().len().hash(state);
        for byte in self.canonical_bytes() {
            state.write_u8(byte);
        }
    }
}

impl fmt::Debug for LegacyIdentifier<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyIdentifier")
            .field("bytes", &self.as_bytes().len())
            .field("case_folded", &self.is_case_folded())
            .finish_non_exhaustive()
    }
}
