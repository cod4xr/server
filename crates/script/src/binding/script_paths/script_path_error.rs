// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::binding::script_paths::ScriptPathUsage;

/// Named virtual-name resource, independent of host/legacy filename limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptPathResource {
    /// Complete input length.
    InputBytes,
    /// Canonical payload length.
    CanonicalBytes,
    /// Validation/scanning/output visits.
    WorkSteps,
}

/// Sanitized virtual-name rejection, without input bytes, names or host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptPathErrorKind {
    /// Catalog names reject NUL instead of silently selecting a C-string prefix.
    EmbeddedNul,
    /// Native non-ASCII ctype behavior is outside the selected safe ASCII profile.
    NonAscii,
    /// Explicit configured resource admission failed.
    BudgetExceeded {
        /// Rejected resource.
        resource: ScriptPathResource,
        /// Requested logical count.
        requested: u64,
        /// Configured maximum.
        maximum: u64,
    },
    /// Checked work/storage arithmetic overflowed.
    SizeOverflow,
    /// Fallible owned-payload reservation failed.
    AllocationFailed,
}

/// Failed operation with preserved admitted work; publishes no partial name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptPathError {
    /// Fixed rejection category.
    pub kind: ScriptPathErrorKind,
    /// Logical work/storage reached before failure.
    pub usage: ScriptPathUsage,
}

impl fmt::Display for ScriptPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "script virtual-name operation failed: {:?}",
            self.kind
        )
    }
}
impl Error for ScriptPathError {}
