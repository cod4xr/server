// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::HeapWorkUsage;

/// Successful managed-array operation with admitted logical work/scratch.
/// Copying a result/handle does not create another owned value location.
#[derive(Clone, Copy, Debug)]
pub struct ArrayResult<T> {
    /// Result data, without implicit ownership beyond the operation's contract.
    pub value: T,
    /// Charged work and peak logical staging scratch.
    pub usage: HeapWorkUsage,
}

/// Outcome of selecting a managed array value's writable data identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrayCowOutcome {
    /// The selected location already has unique data.
    Unchanged,
    /// The location now owns a fresh independent copy of its old data.
    Detached,
}
