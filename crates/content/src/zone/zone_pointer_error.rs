// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Rejected serialized offset/range; no native address or content is included.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZonePointerError {
    /// Four-bit block tag is outside the IW3 profile's nine declared blocks.
    InvalidBlock,
    /// Offset plus requested bytes exceeds the declared block.
    BlockBounds,
    /// Checked offset/size arithmetic overflowed.
    SizeOverflow,
}

impl fmt::Display for ZonePointerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidBlock => "invalid zone stream block",
            Self::BlockBounds => "zone address exceeds declared block",
            Self::SizeOverflow => "zone address size overflow",
        })
    }
}

impl Error for ZonePointerError {}
