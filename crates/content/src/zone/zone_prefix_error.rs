// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Logical metadata resource rejected before publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZonePrefixResource {
    /// Number of script-string slots.
    ScriptStrings,
    /// Number of root asset records.
    Assets,
    /// One inline string, including NUL.
    StringBytes,
    /// Summed inline string bytes, including NULs.
    TotalStringBytes,
    /// Owned string-slot metadata allocation.
    MetadataBytes,
}

/// Sanitized root-metadata errors; no names or script bytes are included.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ZonePrefixError {
    /// Root, pointer table, inline string, or asset directory is incomplete.
    Truncated,
    /// A signed legacy count is negative.
    InvalidCount,
    /// Nonzero count has a null root table pointer.
    MissingTable,
    /// Root metadata would exceed the declared virtual stream block.
    BlockBounds,
    /// A configurable metadata budget was exceeded.
    BudgetExceeded(ZonePrefixResource),
    /// Checked size arithmetic or host representation failed.
    SizeOverflow,
    /// Fallible metadata allocation failed.
    AllocationFailed,
}

impl fmt::Display for ZonePrefixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => formatter.write_str("truncated zone prefix"),
            Self::InvalidCount => formatter.write_str("negative zone root count"),
            Self::MissingTable => formatter.write_str("missing zone root table"),
            Self::BlockBounds => formatter.write_str("zone prefix exceeds virtual block"),
            Self::BudgetExceeded(resource) => write!(formatter, "zone prefix budget: {resource:?}"),
            Self::SizeOverflow => formatter.write_str("zone prefix size cannot be represented"),
            Self::AllocationFailed => formatter.write_str("zone prefix allocation failed"),
        }
    }
}

impl Error for ZonePrefixError {}

pub(super) fn check(
    resource: ZonePrefixResource,
    size: u64,
    limit: u64,
) -> Result<(), ZonePrefixError> {
    if size > limit {
        return Err(ZonePrefixError::BudgetExceeded(resource));
    }
    Ok(())
}
