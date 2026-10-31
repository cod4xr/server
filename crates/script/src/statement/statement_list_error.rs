// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Body-list admission resource, independent of later binding/execution budgets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatementListResource {
    /// Complete retained immediate statement/label count.
    Statements,
    /// Actual visits across both immutable spine passes.
    Steps,
    /// Logical retained node-index bytes.
    MetadataBytes,
}

/// Fixed body-list failure without source contents, labels, names, or paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatementListErrorKind {
    /// The supplied node is not a body-list production.
    NotStatementList,
    /// A required pinned child/production role differs from this profile.
    ProfileMismatch,
    /// Checked count/storage/address arithmetic overflowed.
    SizeOverflow,
    /// Fallible complete index allocation failed.
    AllocationFailed,
    /// A configured resource was exhausted.
    BudgetExceeded {
        /// Resource being admitted.
        resource: StatementListResource,
        /// Requested count/work/bytes.
        requested: u64,
        /// Configured admission budget.
        maximum: u64,
    },
}

/// Rejected body-list admission, retaining failed work rather than resetting quotas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatementListError {
    /// Fixed category/resource counts.
    pub kind: StatementListErrorKind,
    /// Supplied list's original start byte.
    pub offset: u64,
    /// Actual admitted spine visits before failure; rejected work is not charged.
    pub work_steps: u64,
    /// Immediate statements admitted during preflight, not a partial output list.
    pub admitted_statements: u64,
    /// Logical index bytes admitted during preflight, not allocator/RSS usage.
    pub admitted_metadata_bytes: u64,
}

impl fmt::Display for StatementListError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "legacy body-list error at byte {} after {} work steps: {:?}",
            self.offset, self.work_steps, self.kind
        )
    }
}

impl Error for StatementListError {}

pub(super) fn check(
    resource: StatementListResource,
    requested: u64,
    maximum: u64,
) -> Result<(), StatementListErrorKind> {
    if requested > maximum {
        return Err(StatementListErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        });
    }
    Ok(())
}
