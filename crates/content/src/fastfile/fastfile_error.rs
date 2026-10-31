// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// Rejected fastfile admission resource, without payload/name values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FastFileResource {
    /// Complete encoded file bytes.
    FileBytes,
    /// Declared decoded payload bytes.
    PayloadBytes,
    /// One stream block's declared allocation.
    BlockBytes,
    /// Sum of declared block allocations.
    TotalBlockBytes,
    /// Declared external-data accounting.
    ExternalBytes,
}

/// Sanitized envelope failure; no asset names, scripts, or encoded data is included.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FastFileError {
    /// The complete outer header is unavailable.
    Truncated,
    /// The magic bytes do not identify a supported fastfile envelope.
    InvalidMagic,
    /// The source baseline has no loader for this version.
    UnsupportedVersion,
    /// Signed/authenticated envelopes require a different verification boundary.
    UnsupportedAuthentication,
    /// Zlib headers/data/checksum/termination are corrupt or incomplete.
    Compression,
    /// Actual output does not equal the declared header plus payload sizes.
    SizeMismatch,
    /// Encoded data remains after the single complete zlib stream.
    TrailingData,
    /// A configured logical resource limit was exceeded.
    BudgetExceeded(FastFileResource),
    /// Size cannot be represented or checked arithmetic overflowed.
    SizeOverflow,
    /// Fallible decoded output allocation failed.
    AllocationFailed,
}

impl fmt::Display for FastFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => formatter.write_str("truncated fastfile header"),
            Self::InvalidMagic => formatter.write_str("invalid fastfile identifier"),
            Self::UnsupportedVersion => formatter.write_str("unsupported fastfile version"),
            Self::UnsupportedAuthentication => {
                formatter.write_str("authenticated fastfiles are unsupported")
            }
            Self::Compression => formatter.write_str("invalid fastfile zlib stream"),
            Self::SizeMismatch => formatter.write_str("fastfile output size mismatch"),
            Self::TrailingData => formatter.write_str("trailing data after fastfile zlib stream"),
            Self::BudgetExceeded(resource) => {
                write!(formatter, "fastfile budget exceeded: {resource:?}")
            }
            Self::SizeOverflow => formatter.write_str("fastfile size cannot be represented"),
            Self::AllocationFailed => formatter.write_str("fastfile allocation failed"),
        }
    }
}

impl Error for FastFileError {}
