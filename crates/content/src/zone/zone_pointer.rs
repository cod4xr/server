// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::zone::{ZoneAddress, ZonePointerError};

/// IW3 encoded pointer classification; interpretation remains field/type specific.
///
/// Direct offsets and aliases share an encoding. The owning schema decides whether
/// an offset names data or a pointer slot, and which inline sentinels are allowed.
/// This type never casts to a host pointer or resolves a target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZonePointer {
    /// Encoded zero.
    Null,
    /// Encoded -1: ordinary inline data where the schema permits it.
    Inline,
    /// Encoded -2: inline asset plus an inserted alias slot where permitted.
    InlineWithAlias,
    /// Encoded non-sentinel block-relative address, with no target validation yet.
    Offset(ZoneAddress),
}

impl ZonePointer {
    /// Classifies a raw word without following aliases or admitting a pointee.
    ///
    /// # Errors
    /// Rejects block tags outside this profile. Callers separately enforce field
    /// conventions, target type/alignment, zone ownership, residency, and bounds.
    pub fn decode(raw: u32) -> Result<Self, ZonePointerError> {
        match raw {
            0 => Ok(Self::Null),
            u32::MAX => Ok(Self::Inline),
            value if value == u32::MAX - 1 => Ok(Self::InlineWithAlias),
            value => ZoneAddress::decode(value - 1).map(Self::Offset),
        }
    }
}
