// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::binding::script_paths::LegacyScriptPath;

/// Owned canonical virtual identifier, not a permission to open a host filename.
/// Native normalization preserves dots/spaces/trailing slash; filesystem admission
/// and script-file loading remain separate. Payload never appears in Debug.
#[derive(Eq, PartialEq)]
pub struct CanonicalScriptPath {
    pub(super) bytes: Vec<u8>,
}

impl CanonicalScriptPath {
    /// Borrows the complete canonical payload without a native NUL terminator.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Borrows a validated lookup view without allocation, validation or new authority.
    #[must_use]
    pub fn as_path(&self) -> LegacyScriptPath<'_> {
        LegacyScriptPath {
            raw: &self.bytes,
            canonical_len: self.bytes.len(),
        }
    }
}

impl Hash for CanonicalScriptPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_path().hash(state);
    }
}

impl fmt::Debug for CanonicalScriptPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalScriptPath")
            .field("length", &self.bytes.len())
            .finish()
    }
}
