// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::heap::{HeapErrorKind, HeapWorkUsage};

/// Separately admitted shared-object resource; none is an inherited engine ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectResource {
    /// Live shared objects.
    LiveObjects,
    /// Live fields.
    LiveFields,
    /// Live owned field-key bytes.
    KeyBytes,
    /// Retained object slots.
    ObjectSlots,
    /// Live explicitly owned array value locations.
    ArrayValues,
    /// Retained array value arena slots.
    ArrayValueSlots,
    /// Retained field slots.
    FieldSlots,
    /// Retained field-index buckets.
    IndexBuckets,
    /// Logical retained arena/index metadata.
    MetadataBytes,
    /// One complete field key.
    SingleKeyBytes,
    /// Logical visits, probes, copies and possible relocations.
    WorkSteps,
    /// Simultaneous logical staging/mark/root records.
    ScratchBytes,
}

/// Sanitized shared-object failure without keys, values, identities or addresses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectHeapErrorKind {
    /// A field operation or ordinary struct reference received an array data node.
    ExpectedStruct,
    /// An array data operation received an ordinary struct node.
    ExpectedArray,
    /// The managed array value belongs to a different value-location arena.
    ForeignArrayValue,
    /// The managed array value location is missing, vacant, retired or reused.
    StaleArrayValue,
    /// The object belongs to a different or dropped/recreated store.
    ForeignReference,
    /// The object slot is absent, vacant, retired or has a stale generation.
    StaleReference,
    /// Fresh store identities are exhausted and cannot wrap/reuse.
    IdentityExhausted,
    /// This object store is associated with a different string-store identity.
    ForeignStringHeap,
    /// Checked string storage rejected a supplied/stored reference or collection.
    StringHeap(HeapErrorKind),
    /// A configured limit rejected admission before publication/deletion.
    BudgetExceeded {
        /// Resource being admitted.
        resource: ObjectResource,
        /// Requested logical count or bytes.
        requested: u64,
        /// Configured operator budget.
        maximum: u64,
    },
    /// Checked count/storage arithmetic overflowed.
    SizeOverflow,
    /// Fallible arena/index/key/mark/root allocation failed.
    AllocationFailed,
    /// Private arena/index/link/accounting invariants disagreed.
    InvalidState,
}

/// Object operation failure retaining all admitted logical work/scratch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectHeapError {
    /// Fixed private cause/resource counts.
    pub kind: ObjectHeapErrorKind,
    /// Charged work and peak admitted scratch before failure.
    pub usage: HeapWorkUsage,
}

impl fmt::Display for ObjectHeapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "object heap error after {} logical work steps: {:?}",
            self.usage.steps, self.kind
        )
    }
}

impl Error for ObjectHeapError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

impl fmt::Display for ObjectHeapErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ObjectHeapErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::StringHeap(error) = self {
            Some(error)
        } else {
            None
        }
    }
}

pub(super) fn check(
    resource: ObjectResource,
    requested: u64,
    maximum: u64,
) -> Result<(), ObjectHeapErrorKind> {
    if requested > maximum {
        return Err(ObjectHeapErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        });
    }
    Ok(())
}
