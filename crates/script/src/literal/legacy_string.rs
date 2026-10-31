// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::literal::legacy_literal_error;
use crate::literal::{LegacyLiteralError, LegacyLiteralResource, LegacyStringError};

/// Legacy string literal category, independent of runtime interning/handles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyStringKind {
    /// Ordinary quoted string.
    Plain,
    /// Localized &-prefixed quoted string.
    Localized,
}

/// Owned decoded byte string without an implicit terminator or string-table ID.
pub struct LegacyString {
    kind: LegacyStringKind,
    bytes: Vec<u8>,
}

impl LegacyString {
    /// Returns ordinary/localized category without registering a runtime value.
    #[must_use]
    pub fn kind(&self) -> LegacyStringKind {
        self.kind
    }

    /// Borrows exact decoded bytes, without assuming UTF-8 or logging them.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Transfers the owned decoded payload without cloning it.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Concatenates ordinary decoded strings under an explicit result-byte budget.
    ///
    /// This preserves valid native plain-string byte results and intentionally
    /// expands past its fixed scratch-buffer ceiling. No terminator, UTF-8 repair,
    /// formatting/coercion, native interning, or VM reference effects are added.
    /// The budget covers logical result bytes; inputs, allocator overhead, and
    /// aggregate runtime/temporary memory accounting remain caller responsibilities.
    ///
    /// # Errors
    /// Rejects localized operands, checked size overflow, budget exhaustion, and
    /// allocation failure. Inputs remain unchanged and no partial result escapes.
    pub fn concatenate(&self, other: &Self, maximum: u64) -> Result<Self, LegacyStringError> {
        if self.kind != LegacyStringKind::Plain || other.kind != LegacyStringKind::Plain {
            return Err(LegacyStringError::ExpectedPlainString);
        }
        let length = self
            .bytes
            .len()
            .checked_add(other.bytes.len())
            .ok_or(LegacyStringError::SizeOverflow)?;
        if length as u64 > maximum {
            return Err(LegacyStringError::BudgetExceeded {
                requested: length as u64,
                maximum,
            });
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| LegacyStringError::AllocationFailed)?;
        bytes.extend_from_slice(&self.bytes);
        bytes.extend_from_slice(&other.bytes);
        Ok(Self {
            kind: LegacyStringKind::Plain,
            bytes,
        })
    }

    pub(super) fn decode(
        encoded: &[u8],
        kind: LegacyStringKind,
        maximum: u64,
    ) -> Result<Self, LegacyLiteralError> {
        let prefix = match kind {
            LegacyStringKind::Plain => b"\"".as_slice(),
            LegacyStringKind::Localized => b"&\"".as_slice(),
        };
        let body = encoded
            .strip_prefix(prefix)
            .and_then(|bytes| bytes.strip_suffix(b"\""))
            .ok_or(LegacyLiteralError::InvalidSpelling)?;
        let length = decoded(body).count();
        legacy_literal_error::check(LegacyLiteralResource::StringBytes, length as u64, maximum)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| LegacyLiteralError::AllocationFailed)?;
        bytes.extend(decoded(body));
        Ok(Self { kind, bytes })
    }
}

fn decoded(bytes: &[u8]) -> impl Iterator<Item = u8> + '_ {
    let mut input = bytes.iter().copied();
    std::iter::from_fn(move || {
        let byte = input.next()?;
        if byte != b'\\' {
            return Some(byte);
        }
        // The original actions recognize r/t/n and otherwise drop the escape
        // slash. A final unpaired slash ends decoding; valid lexer tokens normally
        // exclude that case. No Unicode/hex/octal or modern escape syntax is added.
        input.next().map(|escaped| match escaped {
            b'r' => b'\r',
            b't' => b'\t',
            b'n' => b'\n',
            byte => byte,
        })
    })
}

impl fmt::Debug for LegacyString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyString")
            .field("kind", &self.kind)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
