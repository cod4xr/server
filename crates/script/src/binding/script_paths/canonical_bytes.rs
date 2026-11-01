// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Allocation-free canonical-byte iterator over a validated ASCII virtual name.
/// Debug hides payloads; iteration is linear in complete raw input visits.
#[derive(Clone)]
pub struct CanonicalScriptPathBytes<'a> {
    raw: &'a [u8],
    boundary: bool,
}

impl<'a> CanonicalScriptPathBytes<'a> {
    pub(super) fn new(raw: &'a [u8]) -> Self {
        Self {
            raw,
            boundary: true,
        }
    }
}

impl Iterator for CanonicalScriptPathBytes<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((byte, rest)) = self.raw.split_first() {
            self.raw = rest;
            if *byte < b' ' {
                self.boundary = true;
                continue;
            }
            if matches!(byte, b'/' | b'\\') {
                if self.boundary {
                    continue;
                }
                self.boundary = true;
                return Some(b'/');
            }
            self.boundary = false;
            return Some(byte.to_ascii_lowercase());
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.raw.len()))
    }
}

impl fmt::Debug for CanonicalScriptPathBytes<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CanonicalScriptPathBytes(..)")
    }
}
