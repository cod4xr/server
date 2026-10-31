// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::binding::LegacyIdentifierError;

/// Exhausted local binding resource, not a native handle/capacity constant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolResource {
    /// Local function definitions.
    Functions,
    /// Total borrowed name bytes.
    NameBytes,
    /// One borrowed name's length.
    SingleNameBytes,
    /// Logical item/name work.
    Steps,
    /// Logical retained index storage.
    MetadataBytes,
}

/// Sanitized local symbol failure, excluding names, file paths, and script text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolErrorKind {
    /// Name cannot enter the explicit legacy identifier profile.
    Identifier(LegacyIdentifierError),
    /// A second local function has the same legacy canonical name.
    DuplicateFunction,
    /// A developer begin marker appears inside an existing developer region.
    NestedDeveloperRegion,
    /// A developer end marker has no opening marker.
    UnexpectedDeveloperEnd,
    /// EOF leaves an opening developer region unclosed.
    UnclosedDeveloperRegion,
    /// Animation-tree directives cannot appear inside developer regions.
    AnimTreeInDeveloperRegion,
    /// A configured resource was exhausted.
    BudgetExceeded(SymbolResource),
    /// A private index invariant failed.
    InvalidState,
    /// Checked size/count arithmetic overflowed.
    SizeOverflow,
    /// Fallible index allocation failed.
    AllocationFailed,
}

/// Local binding failure at original source byte offsets, without received text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SymbolError {
    /// Fixed failure category.
    pub kind: SymbolErrorKind,
    /// Current failing definition/marker or opening marker left unclosed.
    pub offset: u64,
    /// Earlier duplicate definition or enclosing developer marker, when relevant.
    pub previous_offset: Option<u64>,
}

impl fmt::Display for SymbolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "legacy module symbol error at byte {}: {:?}",
            self.offset, self.kind
        )
    }
}

impl Error for SymbolError {}

pub(super) fn check(
    resource: SymbolResource,
    count: u64,
    maximum: u64,
) -> Result<(), SymbolErrorKind> {
    if count > maximum {
        return Err(SymbolErrorKind::BudgetExceeded(resource));
    }
    Ok(())
}
