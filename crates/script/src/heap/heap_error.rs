// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::heap::HeapWorkUsage;

/// Logical string-store resource, never a native packed-pool limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeapResource {
    /// Live string values, including zero-byte strings.
    LiveStrings,
    /// Sum of live owned payload bytes.
    PayloadBytes,
    /// Complete retained slot count.
    Slots,
    /// Logical retained slot records.
    MetadataBytes,
    /// One payload's byte length.
    SingleStringBytes,
    /// Operation visits/copy/relocation admission.
    WorkSteps,
    /// Temporary logical collection flags.
    ScratchBytes,
}

/// Fixed failure category without payload bytes, IDs, source names or host addresses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeapErrorKind {
    /// The reference belongs to a different or dropped/recreated store.
    ForeignReference,
    /// The slot is absent, vacant, retired or has a different generation.
    StaleReference,
    /// Fresh store identities are exhausted; identities never wrap/reuse.
    IdentityExhausted,
    /// A configured logical limit rejected admission before publication/deletion.
    BudgetExceeded {
        /// Resource being admitted.
        resource: HeapResource,
        /// Requested logical count or bytes.
        requested: u64,
        /// Operator-supplied maximum.
        maximum: u64,
    },
    /// Checked count/address/storage arithmetic overflowed.
    SizeOverflow,
    /// Fallible payload/slot/mark allocation failed.
    AllocationFailed,
    /// Private slot/free-list/accounting invariants disagreed.
    InvalidState,
}

/// Failed string operation with all admitted work/scratch preserved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeapError {
    /// Fixed sanitized cause.
    pub kind: HeapErrorKind,
    /// Actual charged logical work and admitted scratch before failure.
    pub usage: HeapWorkUsage,
}

impl fmt::Display for HeapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "string heap error after {} logical work steps: {:?}",
            self.usage.steps, self.kind
        )
    }
}

impl Error for HeapError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

impl fmt::Display for HeapErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for HeapErrorKind {}

pub(super) fn check(
    resource: HeapResource,
    requested: u64,
    maximum: u64,
) -> Result<(), HeapErrorKind> {
    if requested > maximum {
        return Err(HeapErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        });
    }
    Ok(())
}
