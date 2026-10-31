// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::str;

use crate::connection::{ConnectError, ConnectField};
use crate::{CommandLine, InfoLimit, InfoString};

/// Borrowed, validated legacy connect fields, without trusting their claims.
///
/// Unknown userinfo is preserved exactly, including future profile fields. No
/// protocol is accepted, challenge authenticated, password checked, or slot
/// allocated by parsing. Debug output excludes all supplied values/credentials.
pub struct ConnectRequest<'message> {
    userinfo: InfoString<'message>,
    protocol: i32,
    challenge: i32,
    qport: u16,
}

impl<'message> ConnectRequest<'message> {
    /// Parses the first two command tokens using original quote/comment rules.
    ///
    /// As upstream does, additional tokens remain uninterpreted. For a received
    /// packet, first use [`CommandLine::read_connectionless`] to apply the legacy
    /// percent normalization. This method never rewrites or logs userinfo.
    ///
    /// # Errors
    /// Rejects malformed/missing/duplicated critical fields and decimal overflow.
    /// Unlike `atoi`, a numeric prefix followed by junk is not accepted. Qports
    /// accept unsigned decimal or the signed-short spelling emitted upstream.
    pub fn parse(line: CommandLine<'message>) -> Result<Self, ConnectError> {
        let mut tokens = line.tokens();
        if !tokens
            .next()
            .is_some_and(|token| token.eq_ignore_ascii_case(b"connect"))
        {
            return Err(ConnectError::UnexpectedCommand);
        }
        let bytes = tokens.next().ok_or(ConnectError::MissingUserinfo)?;
        let userinfo = InfoString::parse(bytes, InfoLimit::Normal)?;
        let protocol = decimal(userinfo, ConnectField::Protocol)?;
        if protocol < 0 {
            return Err(ConnectError::InvalidField(ConnectField::Protocol));
        }
        let challenge = decimal(userinfo, ConnectField::Challenge)?;
        let qport = decimal(userinfo, ConnectField::Qport)?;
        if !(-32768..=65535).contains(&qport) {
            return Err(ConnectError::InvalidField(ConnectField::Qport));
        }
        let bytes = qport.to_le_bytes();
        Ok(Self {
            userinfo,
            protocol,
            challenge,
            qport: u16::from_le_bytes([bytes[0], bytes[1]]),
        })
    }

    /// Returns the requested protocol; this does not select or authorize a profile.
    #[must_use]
    pub fn protocol(&self) -> i32 {
        self.protocol
    }

    /// Returns the echoed challenge. It must be verified before allocating state.
    #[must_use]
    pub fn challenge(&self) -> i32 {
        self.challenge
    }

    /// Returns the canonical raw 16-bit qport shared with channel packet routing.
    #[must_use]
    pub fn qport(&self) -> u16 {
        self.qport
    }

    /// Borrows original validated key/value bytes; these can contain credentials.
    ///
    /// Never log this view. IP identity must come from verified peer addressing,
    /// not a client-supplied `ip`/name/GUID/userinfo field.
    #[must_use]
    pub fn userinfo(&self) -> InfoString<'message> {
        self.userinfo
    }
}

fn decimal(info: InfoString<'_>, field: ConnectField) -> Result<i32, ConnectError> {
    let mut found = None;
    for pair in info.pairs() {
        if pair.key().eq_ignore_ascii_case(field.key()) {
            if found.is_some() {
                return Err(ConnectError::DuplicateField(field));
            }
            found = Some(pair.value());
        }
    }
    let bytes = found.ok_or(ConnectError::MissingField(field))?;
    str::from_utf8(bytes)
        .ok()
        .and_then(|text| text.trim_ascii().parse().ok())
        .ok_or(ConnectError::InvalidField(field))
}

impl fmt::Debug for ConnectRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectRequest")
            .field("userinfo_bytes", &self.userinfo.as_bytes().len())
            .finish_non_exhaustive()
    }
}
