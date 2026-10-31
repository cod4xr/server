// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::InfoError;

/// An interpreted legacy connect field. Unknown userinfo remains uninterpreted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectField {
    /// Requested protocol number; support is decided by admission policy.
    Protocol,
    /// Echoed opaque signed 32-bit server challenge.
    Challenge,
    /// Client-selected 16-bit channel/NAT routing value.
    Qport,
}

impl ConnectField {
    pub(super) fn key(self) -> &'static [u8] {
        match self {
            Self::Protocol => b"protocol",
            Self::Challenge => b"challenge",
            Self::Qport => b"qport",
        }
    }
}

/// Sanitized connect syntax failure, containing no names, credentials, or echoes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectError {
    /// The command token is not `connect` (ASCII case insensitive).
    UnexpectedCommand,
    /// The userinfo token is absent.
    MissingUserinfo,
    /// Userinfo structure or its legacy byte limit is invalid.
    Info(InfoError),
    /// A required interpreted field is absent.
    MissingField(ConnectField),
    /// A numeric field is malformed, overflowing, or outside its wire domain.
    InvalidField(ConnectField),
    /// A required field occurs more than once, including case aliases.
    DuplicateField(ConnectField),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCommand => formatter.write_str("expected legacy connect command"),
            Self::MissingUserinfo => formatter.write_str("connect userinfo is missing"),
            Self::Info(error) => write!(formatter, "invalid connect userinfo: {error}"),
            Self::MissingField(field) => write!(formatter, "connect field missing: {field:?}"),
            Self::InvalidField(field) => write!(formatter, "connect field invalid: {field:?}"),
            Self::DuplicateField(field) => write!(formatter, "connect field duplicated: {field:?}"),
        }
    }
}

impl Error for ConnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Info(error) => Some(error),
            _ => None,
        }
    }
}

impl From<InfoError> for ConnectError {
    fn from(error: InfoError) -> Self {
        Self::Info(error)
    }
}
