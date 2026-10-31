// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;
use std::net::{SocketAddr, SocketAddrV4};

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

/// Required per-process entropy length for the legacy endpoint cookie key.
pub const ENDPOINT_COOKIE_KEY_BYTES: usize = 32;
const DOMAIN: &[u8] = b"cod4x-rust/legacy-endpoint-cookie/v1\0";

/// A cookie key was not the required 32 bytes; no key material is exposed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CookieKeyError;

impl fmt::Display for CookieKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("endpoint cookie requires a 32-byte process key")
    }
}

impl Error for CookieKeyError {}

/// Stateless 32-bit legacy return-path cookies bound to the observed UDP endpoint.
///
/// Uses HMAC-SHA256 with a fixed server-private domain and a fixed four-byte tag.
/// A cookie is a limited anti-spoofing check, not account authentication, transport
/// integrity, or a modern authentication token. Rate limits remain mandatory.
/// One provider is shared by legacy connect and Source-engine query consumers.
pub struct EndpointCookie {
    initial: Hmac<Sha256>,
}

impl EndpointCookie {
    /// Prepares a provider from exactly 32 bytes of operator/application key data.
    ///
    /// Applications must obtain fresh OS entropy once per process, without a
    /// predictable fallback, and own the source key's lifetime. This library does
    /// not obtain entropy, persist keys, or install logging. Replacing the provider
    /// rotates all cookies. There is deliberately no implicit expiry/clock policy.
    ///
    /// # Errors
    /// Rejects keys of any other length before preparing the MAC state.
    pub fn try_new(secret: &[u8]) -> Result<Self, CookieKeyError> {
        if secret.len() != ENDPOINT_COOKIE_KEY_BYTES {
            return Err(CookieKeyError);
        }
        let mut initial = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| CookieKeyError)?;
        initial.update(DOMAIN);
        Ok(Self { initial })
    }

    /// Issues the opaque signed decimal echo used by the legacy protocol.
    ///
    /// The value is stable for this key/endpoint and includes the UDP source port.
    /// IPv4-mapped IPv6 addresses share IPv4 identity; native IPv6 binds scope ID
    /// but ignores per-packet flow information. No per-peer allocation occurs.
    #[must_use]
    pub fn issue(&self, endpoint: SocketAddr) -> i32 {
        let bytes = self.for_endpoint(endpoint).finalize().into_bytes();
        i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    /// Checks exactly four tag bytes with the MAC library's comparison routine.
    ///
    /// Never allocate connection state solely because this check succeeds: peer
    /// profile, admission policy, resource limits, and authentication are separate.
    #[must_use]
    pub fn verify(&self, endpoint: SocketAddr, challenge: i32) -> bool {
        self.for_endpoint(endpoint)
            .verify_truncated_left(&challenge.to_le_bytes())
            .is_ok()
    }

    fn for_endpoint(&self, endpoint: SocketAddr) -> Hmac<Sha256> {
        let mut mac = self.initial.clone();
        match endpoint {
            SocketAddr::V4(address) => update_ipv4(&mut mac, address),
            SocketAddr::V6(address) => {
                if let Some(ipv4) = address.ip().to_ipv4_mapped() {
                    update_ipv4(&mut mac, SocketAddrV4::new(ipv4, address.port()));
                } else {
                    mac.update(&[6]);
                    mac.update(&address.ip().octets());
                    mac.update(&address.port().to_be_bytes());
                    mac.update(&address.scope_id().to_be_bytes());
                }
            }
        }
        mac
    }
}

fn update_ipv4(mac: &mut Hmac<Sha256>, address: SocketAddrV4) {
    mac.update(&[4]);
    mac.update(&address.ip().octets());
    mac.update(&address.port().to_be_bytes());
}

impl fmt::Debug for EndpointCookie {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EndpointCookie { .. }")
    }
}
