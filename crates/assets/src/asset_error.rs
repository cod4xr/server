// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

/// An explicit catalog budget category suitable for low-cardinality diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AssetResource {
    /// Allocated metadata slots.
    Slots,
    /// Total encoded payload bytes.
    PayloadBytes,
    /// Encoded bytes in one asset.
    SingleAssetBytes,
    /// Bytes in one identifier.
    NameBytes,
    /// Total bytes in the ownership/index identifier copies.
    TotalNameBytes,
}

/// Asset ownership failures without names, contents, or private configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AssetError {
    /// An empty name cannot identify an asset.
    EmptyName,
    /// Embedded NUL is not a supported asset identifier.
    EmbeddedTerminator,
    /// The exact kind/name is already registered; existing contents are unchanged.
    DuplicateName,
    /// A specific ownership budget cannot accept the complete operation.
    BudgetExceeded {
        /// Budget that rejected the operation.
        resource: AssetResource,
        /// Complete newly requested units.
        requested: u64,
        /// Units available for this operation.
        available: u64,
    },
    /// An allocation/capacity reservation failed before publishing an asset.
    AllocationFailed,
    /// A count cannot be represented without overflow.
    SizeOverflow,
    /// Catalog identity namespace is exhausted; IDs must never wrap.
    IdentityExhausted,
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => formatter.write_str("asset name is empty"),
            Self::EmbeddedTerminator => formatter.write_str("asset name contains NUL"),
            Self::DuplicateName => formatter.write_str("asset kind/name is already registered"),
            Self::BudgetExceeded {
                resource,
                requested,
                available,
            } => {
                write!(
                    formatter,
                    "asset budget {resource:?} needs {requested}; {available} available"
                )
            }
            Self::AllocationFailed => formatter.write_str("asset allocation reservation failed"),
            Self::SizeOverflow => formatter.write_str("asset size cannot be represented"),
            Self::IdentityExhausted => formatter.write_str("catalog identity namespace exhausted"),
        }
    }
}

impl Error for AssetError {}
