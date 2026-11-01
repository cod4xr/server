// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::legacy::{LegacyNumber, LegacyVector};

/// Numeric/vector operand subset, without strings, objects, references or authority.
#[derive(Clone, Copy)]
pub enum LegacyVectorOperand {
    /// Number eligible for broadcast when its peer is a vector.
    Number(LegacyNumber),
    /// Owned fixed-size vector data.
    Vector(LegacyVector),
}

impl fmt::Debug for LegacyVectorOperand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Number(_) => "LegacyVectorOperand::Number(..)",
            Self::Vector(_) => "LegacyVectorOperand::Vector(..)",
        })
    }
}
