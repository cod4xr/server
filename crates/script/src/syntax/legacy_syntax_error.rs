// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::lexical::LegacyLexErrorKind;

/// Exhausted structural parser resource; not a legacy engine capacity constant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacySyntaxResource {
    /// Flat tree nodes.
    Nodes,
    /// Parent-child edges.
    ChildEdges,
    /// Live parser stack entries.
    StackEntries,
    /// Grammar action attempts.
    Steps,
    /// Logical live decode metadata bytes.
    MetadataBytes,
}

/// Sanitized structural failure without source strings, names, or host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacySyntaxErrorKind {
    /// Scanner failed before this grammar could consume another terminal.
    Lexical(LegacyLexErrorKind),
    /// The grammar has no valid action for the current source/EOF.
    UnexpectedToken,
    /// A configured structural resource was exhausted.
    BudgetExceeded(LegacySyntaxResource),
    /// A generated table/stack/tree invariant failed.
    InvalidTable,
    /// Checked native size or logical byte arithmetic overflowed.
    SizeOverflow,
    /// Fallible tree/edge/stack allocation failed.
    AllocationFailed,
}

/// Structural failure at a zero-based source byte offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacySyntaxError {
    /// Fixed failure category, excluding raw source contents.
    pub kind: LegacySyntaxErrorKind,
    /// Offending terminal/EOF or scanner error byte offset.
    pub offset: u64,
}

impl fmt::Display for LegacySyntaxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "legacy structural syntax error at byte {}: {:?}",
            self.offset, self.kind
        )
    }
}

impl Error for LegacySyntaxError {}

pub(super) fn check(
    resource: LegacySyntaxResource,
    size: u64,
    maximum: u64,
) -> Result<(), LegacySyntaxErrorKind> {
    if size > maximum {
        return Err(LegacySyntaxErrorKind::BudgetExceeded(resource));
    }
    Ok(())
}
