// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::binding::script_paths::script_path_work::ScriptPathWork;
use crate::binding::script_paths::{
    CanonicalScriptPath, CanonicalScriptPathBytes, ScriptPathBudget, ScriptPathError,
    ScriptPathErrorKind, ScriptPathResult,
};

/// Validated NUL-free ASCII script-name view under the native canonicalizer profile.
///
/// Separators collapse, leading separators/control bytes disappear, and ASCII folds
/// to lowercase. Control bytes restart the helper's segment state; dots/spaces and
/// a single trailing separator remain. This is a virtual identity, not filesystem
/// validation, script loading, an interner ID or full non-ASCII native behavior.
/// Hash/equality/iteration scan raw bytes without allocation; a linker must prepay
/// those visits separately from construction. No native 64/1,024-byte ceiling applies.
#[derive(Clone, Copy)]
pub struct LegacyScriptPath<'a> {
    pub(super) raw: &'a [u8],
    pub(super) canonical_len: usize,
}

impl<'a> LegacyScriptPath<'a> {
    /// Validates the complete input and counts canonical bytes without allocating.
    /// NUL rejects rather than truncating; non-ASCII rejects outside the safe profile.
    /// # Errors
    /// Invalid bytes and input/output/work limits publish no partial view. Count/
    /// work overflow returns a typed error; admitted failed work is preserved.
    pub fn new(
        raw: &'a [u8],
        budget: ScriptPathBudget,
    ) -> Result<ScriptPathResult<Self>, ScriptPathError> {
        let mut work = ScriptPathWork::new(raw.len(), budget)?;
        for byte in raw {
            work.charge(1)?;
            if *byte == 0 {
                return Err(work.error(ScriptPathErrorKind::EmbeddedNul));
            }
            if !byte.is_ascii() {
                return Err(work.error(ScriptPathErrorKind::NonAscii));
            }
        }
        work.charge(raw.len() as u64)?;
        let mut count = 0_usize;
        for _ in CanonicalScriptPathBytes::new(raw) {
            count = count
                .checked_add(1)
                .ok_or_else(|| work.error(ScriptPathErrorKind::SizeOverflow))?;
            work.canonical(count)?;
        }
        Ok(ScriptPathResult {
            value: Self {
                raw,
                canonical_len: count,
            },
            usage: work.usage,
        })
    }

    /// Returns complete canonical length without rescanning or granting ownership.
    #[must_use]
    pub fn canonical_len(self) -> u64 {
        self.canonical_len as u64
    }

    /// Borrows a fresh canonical-byte iterator; it does not allocate or charge work.
    #[must_use]
    pub fn canonical_bytes(self) -> CanonicalScriptPathBytes<'a> {
        CanonicalScriptPathBytes::new(self.raw)
    }

    /// Owns canonical bytes after complete admission and a fallible reservation.
    /// The result survives source destruction. Fixed fields, spare capacity and
    /// allocator overhead are excluded from retained payload accounting.
    /// # Errors
    /// Input/output/work limits, overflow or allocation publish no partial owned name.
    pub fn copy_owned(
        self,
        budget: ScriptPathBudget,
    ) -> Result<ScriptPathResult<CanonicalScriptPath>, ScriptPathError> {
        let mut work = ScriptPathWork::new(self.raw.len(), budget)?;
        work.canonical(self.canonical_len)?;
        work.charge(self.raw.len() as u64)?;
        work.charge(self.canonical_len as u64)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(self.canonical_len)
            .map_err(|_| work.error(ScriptPathErrorKind::AllocationFailed))?;
        bytes.extend(self.canonical_bytes());
        work.usage.retained_bytes = self.canonical_len as u64;
        Ok(ScriptPathResult {
            value: CanonicalScriptPath { bytes },
            usage: work.usage,
        })
    }
}

impl PartialEq for LegacyScriptPath<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_len == other.canonical_len
            && self.canonical_bytes().eq(other.canonical_bytes())
    }
}
impl Eq for LegacyScriptPath<'_> {}
impl Hash for LegacyScriptPath<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.canonical_len.hash(state);
        for byte in self.canonical_bytes() {
            byte.hash(state);
        }
    }
}
impl fmt::Debug for LegacyScriptPath<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyScriptPath")
            .field("input_bytes", &self.raw.len())
            .field("canonical_bytes", &self.canonical_len)
            .finish()
    }
}
