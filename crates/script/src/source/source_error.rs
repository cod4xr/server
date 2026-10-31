// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Exhausted source-index resource, excluding source text and host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceResource {
    /// Borrowed source byte count.
    SourceBytes,
    /// Number of indexed lines.
    Lines,
    /// Logical owned line-index bytes.
    MetadataBytes,
}

/// Sanitized source admission/lookup failure with no source contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceError {
    /// Byte offset is past EOF or cannot be represented by the host.
    InvalidOffset,
    /// A one-based line number is zero or outside this source.
    InvalidLine,
    /// A requested byte span is reversed, out of range, or unrepresentable.
    InvalidSpan,
    /// A configured logical resource budget was exceeded.
    BudgetExceeded(SourceResource),
    /// Checked metadata arithmetic overflowed.
    SizeOverflow,
    /// Fallible line-index allocation failed.
    AllocationFailed,
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOffset => formatter.write_str("invalid script source offset"),
            Self::InvalidLine => formatter.write_str("invalid script source line"),
            Self::InvalidSpan => formatter.write_str("invalid script source span"),
            Self::BudgetExceeded(resource) => {
                write!(formatter, "script source budget: {resource:?}")
            }
            Self::SizeOverflow => formatter.write_str("script source index overflow"),
            Self::AllocationFailed => formatter.write_str("script source index allocation failed"),
        }
    }
}

impl Error for SourceError {}

pub(super) fn check(resource: SourceResource, size: u64, maximum: u64) -> Result<(), SourceError> {
    if size > maximum {
        return Err(SourceError::BudgetExceeded(resource));
    }
    Ok(())
}
