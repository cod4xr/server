// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Rejected compression work, containing no source or destination data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompressionError {
    /// Input or produced logical message exceeds this codec's 128 KiB profile.
    MessageTooLarge,
    /// The caller cannot hold the complete encoded/decoded result.
    OutputCapacity,
    /// The reserved "not yet transmitted" symbol is absent from normal messages.
    ReservedSymbol,
    /// A bit pattern cannot be resolved within the fixed codebook's maximum depth.
    InvalidCode,
}

impl fmt::Display for CompressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MessageTooLarge => "compression exceeds the compatibility message budget",
            Self::OutputCapacity => "compression output storage is insufficient",
            Self::ReservedSymbol => "compressed input contains a reserved symbol",
            Self::InvalidCode => "compressed input contains an invalid code",
        };
        formatter.write_str(message)
    }
}

impl Error for CompressionError {}
