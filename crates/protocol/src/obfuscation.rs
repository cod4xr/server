// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy channel XOR compatibility. This provides no encryption/authentication.
//!
//! Functions operate only on the transformed payload. Callers parse and preserve
//! the profile-specific plaintext acknowledgement header separately. Applying the
//! same transform twice restores the original bytes, in either peer direction.

mod keyed_payload;
mod obfuscation_error;

pub use keyed_payload::{apply_client_payload, apply_server_payload};
pub use obfuscation_error::ObfuscationError;
