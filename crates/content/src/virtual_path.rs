// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::{ArchiveError, ArchiveResource};

/// Validated relative byte path with ASCII case folding and `/` separators.
///
/// This is an archive namespace, never permission to access a host filesystem.
/// High bytes remain exact. Empty/dot/parent components, colon, control bytes,
/// absolute paths, and repeated/trailing separators are rejected, not repaired.
#[derive(Eq, Hash, PartialEq)]
pub struct VirtualPath(Vec<u8>);

impl VirtualPath {
    /// Validates and allocates a canonical archive path under an explicit limit.
    ///
    /// # Errors
    /// Returns invalid-path, path-budget, or allocation errors without exposing
    /// the supplied name. The original bytes remain unchanged.
    pub fn try_new(bytes: &[u8], max_bytes: usize) -> Result<Self, ArchiveError> {
        if bytes.len() > max_bytes {
            return Err(ArchiveError::BudgetExceeded {
                resource: ArchiveResource::PathBytes,
            });
        }
        if bytes.is_empty()
            || bytes
                .iter()
                .any(|byte| byte.is_ascii_control() || *byte == b':')
            || bytes
                .split(|byte| matches!(byte, b'/' | b'\\'))
                .any(|part| part.is_empty() || part == b"." || part == b"..")
        {
            return Err(ArchiveError::InvalidPath);
        }
        let mut canonical = Vec::new();
        canonical
            .try_reserve_exact(bytes.len())
            .map_err(|_| ArchiveError::AllocationFailed)?;
        canonical.extend(bytes.iter().map(|byte| {
            if *byte == b'\\' {
                b'/'
            } else {
                byte.to_ascii_lowercase()
            }
        }));
        Ok(Self(canonical))
    }

    /// Returns the normalized byte name without UTF-8 conversion.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub(super) fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Debug for VirtualPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VirtualPath")
            .field("bytes", &self.0.len())
            .finish()
    }
}
