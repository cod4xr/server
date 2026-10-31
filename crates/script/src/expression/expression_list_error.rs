// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Exhausted frontend list resource, not a runtime argument capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpressionListResource {
    /// Retained expression entries.
    Expressions,
    /// Node visits across both construction passes.
    Steps,
    /// Logical retained scoped-node metadata.
    MetadataBytes,
}

/// Fixed expression-list failure category without script contents or host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpressionListErrorKind {
    /// The supplied node is not a supported expression-list production.
    ExpectedList,
    /// The private tree differs from the pinned list production/child profile.
    InvalidStructure,
    /// A configured resource was exhausted before output allocation.
    BudgetExceeded {
        /// Resource being admitted.
        resource: ExpressionListResource,
        /// Complete requested logical count or bytes at this check.
        requested: u64,
        /// Configured admission budget.
        maximum: u64,
    },
    /// Checked count/storage arithmetic or address-space conversion overflowed.
    SizeOverflow,
    /// Fallible complete index allocation failed.
    AllocationFailed,
}

/// List failure at the original supplied list node's zero-based source byte offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpressionListError {
    /// Fixed category and resource accounting, never source strings/names.
    pub kind: ExpressionListErrorKind,
    /// Original supplied node's start, suitable for separate source indexing.
    pub offset: u64,
}

impl fmt::Display for ExpressionListError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "legacy expression list error at byte {}: {:?}",
            self.offset, self.kind
        )
    }
}

impl Error for ExpressionListError {}

pub(super) fn check(
    resource: ExpressionListResource,
    requested: u64,
    maximum: u64,
) -> Result<(), ExpressionListErrorKind> {
    if requested > maximum {
        return Err(ExpressionListErrorKind::BudgetExceeded {
            resource,
            requested,
            maximum,
        });
    }
    Ok(())
}
