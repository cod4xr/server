// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Exhausted declaration-lowering resource, not a legacy capacity constant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclarationResource {
    /// Top-level function/directive/developer items.
    Items,
    /// Include directives.
    Includes,
    /// Total parameter declarations.
    Parameters,
    /// Node visits during extraction.
    Steps,
    /// Logical retained metadata bytes.
    MetadataBytes,
}

/// Fixed extraction failure category, without names, strings, or host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclarationErrorKind {
    /// Module declarations require Script rather than Fragment grammar entry.
    ExpectedScript,
    /// The pinned tree shape does not match a required declaration invariant.
    InvalidStructure,
    /// A configured resource was exhausted.
    BudgetExceeded(DeclarationResource),
    /// Checked count/storage arithmetic overflowed.
    SizeOverflow,
    /// Fallible item/include/parameter allocation failed.
    AllocationFailed,
}

/// Declaration failure at a zero-based byte offset in the original source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclarationError {
    /// Fixed category; never contains script contents.
    pub kind: DeclarationErrorKind,
    /// Failing node's source byte offset.
    pub offset: u64,
}

impl fmt::Display for DeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "legacy declaration error at byte {}: {:?}",
            self.offset, self.kind
        )
    }
}

impl Error for DeclarationError {}

pub(super) fn check(
    resource: DeclarationResource,
    size: u64,
    maximum: u64,
) -> Result<(), DeclarationErrorKind> {
    if size > maximum {
        return Err(DeclarationErrorKind::BudgetExceeded(resource));
    }
    Ok(())
}
